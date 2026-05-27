use crate::db::{Db, SharedLinkRecord};
use anyhow::Result;
use serde::Serialize;
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct TunnelStatus {
    pub active: bool,
    pub url: Option<String>,
    pub error: Option<String>,
}

pub struct SharingHubManager {
    db: Db,
    tunnel_process: Mutex<Option<tokio::process::Child>>,
    tunnel_url: Mutex<Option<String>>,
}

impl SharingHubManager {
    pub fn new(db: Db) -> Result<Self> {
        Ok(Self {
            db,
            tunnel_process: Mutex::new(None),
            tunnel_url: Mutex::new(None),
        })
    }

    pub fn get_shared_links(&self) -> Result<Vec<SharedLinkRecord>> {
        self.db.get_shared_links()
    }

    pub async fn create_shared_link(
        &self,
        torrent_hash: &str,
        file_path: &str,
        password: Option<String>,
        expiry_mins: Option<i64>,
    ) -> Result<SharedLinkRecord> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let password_hash = match password {
            Some(pw) if !pw.is_empty() => {
                Some(bcrypt::hash(&pw, 10)?)
            }
            _ => None,
        };

        let expiry_at = expiry_mins.map(|mins| {
            (chrono::Utc::now() + chrono::Duration::minutes(mins)).timestamp()
        });

        let record = SharedLinkRecord {
            id,
            torrent_hash: torrent_hash.to_string(),
            file_path: file_path.to_string(),
            password_hash,
            expiry_at,
            created_at: now,
            access_count: 0,
            bandwidth_used_bytes: 0,
        };

        self.db.insert_shared_link(&record)?;
        Ok(record)
    }

    pub fn delete_shared_link(&self, id: &str) -> Result<()> {
        self.db.delete_shared_link(id)
    }

    pub async fn get_tunnel_status(&self) -> TunnelStatus {
        let url = self.tunnel_url.lock().await;
        let process = self.tunnel_process.lock().await;

        if process.is_some() && url.is_some() {
            TunnelStatus {
                active: true,
                url: url.clone(),
                error: None,
            }
        } else {
            TunnelStatus {
                active: false,
                url: None,
                error: None,
            }
        }
    }

    pub async fn start_tunnel(&self, local_port: u16) -> Result<String> {
        let mut process_guard = self.tunnel_process.lock().await;
        let mut url_guard = self.tunnel_url.lock().await;

        if process_guard.is_some() {
            return url_guard
                .clone()
                .ok_or_else(|| anyhow::anyhow!("tunnel running but no URL"));
        }

        let mut child = Command::new("ssh")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("ServerAliveInterval=30")
            .arg("-R")
            .arg(format!("80:localhost:{}", local_port))
            .arg("nokey@localhost.run")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let tunnel_url = self.wait_for_tunnel_url(&mut child).await?;

        *process_guard = Some(child);
        *url_guard = Some(tunnel_url.clone());

        Ok(tunnel_url)
    }

    pub async fn stop_tunnel(&self) -> Result<()> {
        let mut process_guard = self.tunnel_process.lock().await;
        let mut url_guard = self.tunnel_url.lock().await;

        if let Some(mut child) = process_guard.take() {
            let _ = child.kill().await;
        }
        *url_guard = None;
        Ok(())
    }

    async fn wait_for_tunnel_url(&self, child: &mut tokio::process::Child) -> Result<String> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("no stdout from ssh"))?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let timeout = tokio::time::timeout(std::time::Duration::from_secs(30), async {
            while let Ok(Some(line)) = lines.next_line().await {
                if line.contains("localhost.run") || line.contains("https://") {
                    let url = line
                        .split_whitespace()
                        .find(|w| w.starts_with("https://"))
                        .unwrap_or(&line)
                        .trim()
                        .to_string();
                    return Ok(url);
                }
            }
            Err(anyhow::anyhow!("tunnel URL not received"))
        })
        .await??;

        // Put stdout back (we consumed it)
        // Since we can't put it back easily, we'll just leave it
        // The process will continue running
        Ok(timeout)
    }

    pub async fn get_upnp_status() -> Result<UpnpStatus> {
        Ok(UpnpStatus {
            enabled: false,
            external_ip: None,
            mapped_port: None,
            message: "UPnP discovery not yet implemented".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UpnpStatus {
    pub enabled: bool,
    pub external_ip: Option<String>,
    pub mapped_port: Option<u16>,
    pub message: String,
}
