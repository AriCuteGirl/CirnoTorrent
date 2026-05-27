use crate::db::{Db, TorrentRecord};
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct TorrentStatus {
    pub id: String,
    pub info_hash: String,
    pub name: String,
    pub status: String,
    pub progress: f64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub total_bytes: u64,
    pub peers_connected: u32,
    pub seeds_connected: u32,
    pub eta_secs: f64,
    pub category: String,
    pub tags: String,
    pub sequential: bool,
    pub save_path: String,
    pub added_at: String,
    pub files: Vec<FileEntry>,
    pub trackers: Vec<String>,
    pub piece_count: u32,
    pub piece_length: u64,
    pub piece_map: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub index: usize,
    pub path: String,
    pub size: u64,
    pub priority: String,
    pub progress: f64,
}

struct ActiveTorrent {
    handle: Arc<librqbit::ManagedTorrent>,
    librqbit_id: usize,
}

pub struct TorrentManager {
    session: Arc<librqbit::Session>,
    db: Db,
    active: Mutex<HashMap<String, ActiveTorrent>>,
    download_dir: PathBuf,
}

impl TorrentManager {
    pub fn new(db: Db, download_dir: &str) -> Result<Self> {
        let download_path = PathBuf::from(download_dir);
        std::fs::create_dir_all(&download_path).ok();

        let session = {
            let dl = download_path.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to create tokio runtime for librqbit session");
                rt.block_on(async move {
                    librqbit::Session::new(dl).await
                })
            })
            .join()
            .map_err(|_| anyhow::anyhow!("failed to join session thread"))??
        };

        Ok(Self {
            session,
            db,
            active: Mutex::new(HashMap::new()),
            download_dir: download_path,
        })
    }

    pub async fn add_torrent_from_bytes(&self, data: Vec<u8>, category: &str, tags: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let add_torrent = librqbit::AddTorrent::from_bytes(bytes::Bytes::from(data));
        let response = self.session.add_torrent(add_torrent, None).await?;

        self.register_torrent(response, id, now, "", category, tags).await
    }

    pub async fn add_torrent(&self, magnet_or_url: &str, category: &str, tags: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let add_torrent = librqbit::AddTorrent::from_url(magnet_or_url);
        let response = self.session.add_torrent(add_torrent, None).await?;

        self.register_torrent(response, id, now, magnet_or_url, category, tags).await
    }

    async fn register_torrent(
        &self,
        response: librqbit::AddTorrentResponse,
        id: String,
        now: String,
        magnet_uri: &str,
        category: &str,
        tags: &str,
    ) -> Result<String> {
        let handle = response.into_handle()
            .ok_or_else(|| anyhow::anyhow!("torrent was not added (list-only or already managed)"))?;

        let librqbit_id = self.session.with_torrents(|torrents| {
            for (tid, t) in torrents {
                if Arc::ptr_eq(t, &handle) {
                    return Some(tid);
                }
            }
            None
        }).unwrap_or(0);

        let info_hash = hex::encode(handle.info_hash().0);
        let stats = handle.stats();
        let name = format!("torrent-{}", &info_hash[..8]);
        let total_bytes = stats.total_bytes as i64;

        let record = TorrentRecord {
            id: id.clone(),
            info_hash,
            name,
            magnet_uri: magnet_uri.to_string(),
            save_path: self.download_dir.to_string_lossy().to_string(),
            total_bytes,
            downloaded_bytes: 0,
            uploaded_bytes: 0,
            status: "downloading".to_string(),
            category: category.to_string(),
            tags: tags.to_string(),
            sequential: false,
            download_speed_limit: 0,
            upload_speed_limit: 0,
            ratio_limit: 0.0,
            seeding_time_limit_mins: 0,
            added_at: now,
            completed_at: None,
        };

        self.db.insert_torrent(&record)?;

        let mut active = self.active.lock().await;
        active.insert(id.clone(), ActiveTorrent { handle, librqbit_id });

        Ok(id)
    }

    pub async fn pause_torrent(&self, id: &str) -> Result<()> {
        let active = self.active.lock().await;
        if let Some(torrent) = active.get(id) {
            self.session.pause(&torrent.handle).await?;
        }
        drop(active);
        self.db.update_torrent_status(id, "paused")?;
        Ok(())
    }

    pub async fn resume_torrent(&self, id: &str) -> Result<()> {
        let active = self.active.lock().await;
        if let Some(torrent) = active.get(id) {
            self.session.unpause(&torrent.handle).await?;
        }
        drop(active);
        self.db.update_torrent_status(id, "downloading")?;
        Ok(())
    }

    pub async fn remove_torrent(&self, id: &str, delete_files: bool) -> Result<()> {
        let mut active = self.active.lock().await;
        if let Some(torrent) = active.remove(id) {
            self.session.delete(
                librqbit::api::TorrentIdOrHash::Id(torrent.librqbit_id),
                delete_files,
            ).await?;
        }
        drop(active);
        self.db.delete_torrent(id)?;
        Ok(())
    }

    pub async fn set_speed_limits(&self, id: &str, download_kb: i64, upload_kb: i64) -> Result<()> {
        self.db.update_torrent_speed_limits(id, download_kb, upload_kb)?;
        Ok(())
    }

    pub async fn set_torrent_category(&self, id: &str, category: &str) -> Result<()> {
        self.db.update_torrent_category(id, category)?;
        Ok(())
    }

    pub async fn set_torrent_priority(&self, _id: &str, _file_index: usize, _priority: &str) -> Result<()> {
        Ok(())
    }

    pub async fn toggle_sequential_mode(&self, id: &str) -> Result<bool> {
        let record = self.db.get_torrent_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("torrent not found"))?;
        let new_val = !record.sequential;
        self.db.update_torrent_sequential(id, new_val)?;
        Ok(new_val)
    }

    pub async fn get_all_torrent_statuses(&self) -> Vec<TorrentStatus> {
        let records = self.db.get_torrents().unwrap_or_default();
        let active = self.active.lock().await;
        let mut statuses = Vec::new();

        for record in records {
            let (dl_speed, ul_speed, peers, seeds, progress, downloaded, uploaded, total, files, trackers, piece_count, piece_length, piece_map) =
                if let Some(torrent) = active.get(&record.id) {
                    let h = &torrent.handle;
                    let stats = h.stats();

                    let (dl, ul, peers_count) = if let Some(ref live) = stats.live {
                        (
                            (live.download_speed.mbps * 1048576.0) as u64,
                            (live.upload_speed.mbps * 1048576.0) as u64,
                            0u32,
                        )
                    } else {
                        (0u64, 0u64, 0u32)
                    };

                    let total_len = stats.total_bytes;
                    let downloaded_len = stats.progress_bytes;
                    let uploaded_len = stats.uploaded_bytes;
                    let prog = if total_len > 0 {
                        (downloaded_len as f64 / total_len as f64) * 100.0
                    } else {
                        0.0
                    };

                    let file_entries: Vec<FileEntry> = stats.file_progress.iter().enumerate().map(|(idx, &fp)| {
                        FileEntry {
                            index: idx,
                            path: format!("file_{}", idx),
                            size: fp,
                            priority: "normal".to_string(),
                            progress: if fp > 0 { 1.0 } else { 0.0 },
                        }
                    }).collect();

                    let tracker_list: Vec<String> = vec![];
                    let pc = 0u32;
                    let pl = 0u64;
                    let pm: Vec<u8> = vec![];

                    (dl, ul, peers_count, 0u32, prog, downloaded_len, uploaded_len, total_len, file_entries, tracker_list, pc, pl, pm)
                } else {
                    let total = record.total_bytes as u64;
                    let downloaded = record.downloaded_bytes as u64;
                    let prog = if total > 0 {
                        (downloaded as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    };
                    (0u64, 0u64, 0u32, 0u32, prog, downloaded, record.uploaded_bytes as u64, total, vec![], vec![], 0u32, 0u64, vec![])
                };

            let eta = if dl_speed > 0 && total > downloaded {
                (total - downloaded) as f64 / dl_speed as f64
            } else {
                -1.0
            };

            statuses.push(TorrentStatus {
                id: record.id,
                info_hash: record.info_hash,
                name: record.name,
                status: record.status,
                progress,
                download_speed: dl_speed,
                upload_speed: ul_speed,
                downloaded_bytes: downloaded,
                uploaded_bytes: uploaded,
                total_bytes: total,
                peers_connected: peers,
                seeds_connected: seeds,
                eta_secs: eta,
                category: record.category,
                tags: record.tags,
                sequential: record.sequential,
                save_path: record.save_path,
                added_at: record.added_at,
                files,
                trackers,
                piece_count,
                piece_length,
                piece_map,
            });
        }

        statuses
    }
}
