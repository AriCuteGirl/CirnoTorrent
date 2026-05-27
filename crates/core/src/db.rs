use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub downloads_dir: String,
    pub watch_dir: String,
    pub max_download_speed_kb: i64,
    pub max_upload_speed_kb: i64,
    pub max_connections: i32,
    pub max_connections_per_torrent: i32,
    pub dht_enabled: bool,
    pub pex_enabled: bool,
    pub lsd_enabled: bool,
    pub utp_enabled: bool,
    pub listen_port: i32,
    pub webui_enabled: bool,
    pub webui_port: i32,
    pub webui_username: String,
    pub webui_password_hash: String,
    pub auto_extract: bool,
    pub extract_path: String,
    pub extract_passwords: String,
    pub rss_poll_interval_secs: i32,
    pub jackett_url: String,
    pub jackett_api_key: String,
    pub accent_color: String,
    pub theme: String,
    pub sequential_default: bool,
    pub ratio_limit: f64,
    pub seeding_time_limit_mins: i64,
    pub notifications_enabled: bool,
    pub blocklist_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            downloads_dir: format!("{}/Downloads", home),
            watch_dir: String::new(),
            max_download_speed_kb: 0,
            max_upload_speed_kb: 0,
            max_connections: 200,
            max_connections_per_torrent: 50,
            dht_enabled: true,
            pex_enabled: true,
            lsd_enabled: true,
            utp_enabled: true,
            listen_port: 6881,
            webui_enabled: false,
            webui_port: 8080,
            webui_username: "admin".to_string(),
            webui_password_hash: String::new(),
            auto_extract: false,
            extract_path: String::new(),
            extract_passwords: String::new(),
            rss_poll_interval_secs: 900,
            jackett_url: String::new(),
            jackett_api_key: String::new(),
            accent_color: "blue".to_string(),
            theme: "dark".to_string(),
            sequential_default: false,
            ratio_limit: 0.0,
            seeding_time_limit_mins: 0,
            notifications_enabled: true,
            blocklist_url: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentRecord {
    pub id: String,
    pub info_hash: String,
    pub name: String,
    pub magnet_uri: String,
    pub save_path: String,
    pub total_bytes: i64,
    pub downloaded_bytes: i64,
    pub uploaded_bytes: i64,
    pub status: String,
    pub category: String,
    pub tags: String,
    pub sequential: bool,
    pub download_speed_limit: i64,
    pub upload_speed_limit: i64,
    pub ratio_limit: f64,
    pub seeding_time_limit_mins: i64,
    pub added_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRecord {
    pub id: String,
    pub torrent_id: String,
    pub archive_path: String,
    pub output_dir: String,
    pub status: String,
    pub progress: f64,
    pub password: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedLinkRecord {
    pub id: String,
    pub torrent_hash: String,
    pub file_path: String,
    pub password_hash: Option<String>,
    pub expiry_at: Option<i64>,
    pub created_at: String,
    pub access_count: i64,
    pub bandwidth_used_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssFeedRecord {
    pub id: String,
    pub name: String,
    pub url: String,
    pub last_polled_at: Option<String>,
    pub last_etag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssRuleRecord {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub feed_id: Option<String>,
    pub category: String,
    pub save_path: String,
    pub last_matched_at: Option<String>,
}

impl Db {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS torrents (
                id TEXT PRIMARY KEY,
                info_hash TEXT NOT NULL DEFAULT '',
                name TEXT NOT NULL DEFAULT '',
                magnet_uri TEXT NOT NULL DEFAULT '',
                save_path TEXT NOT NULL DEFAULT '',
                total_bytes INTEGER NOT NULL DEFAULT 0,
                downloaded_bytes INTEGER NOT NULL DEFAULT 0,
                uploaded_bytes INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'queued',
                category TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                sequential INTEGER NOT NULL DEFAULT 0,
                download_speed_limit INTEGER NOT NULL DEFAULT 0,
                upload_speed_limit INTEGER NOT NULL DEFAULT 0,
                ratio_limit REAL NOT NULL DEFAULT 0.0,
                seeding_time_limit_mins INTEGER NOT NULL DEFAULT 0,
                added_at TEXT NOT NULL DEFAULT '',
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS rss_feeds (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                last_polled_at TEXT,
                last_etag TEXT
            );

            CREATE TABLE IF NOT EXISTS rss_rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                pattern TEXT NOT NULL,
                feed_id TEXT,
                category TEXT NOT NULL DEFAULT '',
                save_path TEXT NOT NULL DEFAULT '',
                last_matched_at TEXT
            );

            CREATE TABLE IF NOT EXISTS extraction_queue (
                id TEXT PRIMARY KEY,
                torrent_id TEXT NOT NULL,
                archive_path TEXT NOT NULL,
                output_dir TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'pending',
                progress REAL NOT NULL DEFAULT 0.0,
                password TEXT,
                error_message TEXT,
                started_at TEXT NOT NULL DEFAULT '',
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS shared_links (
                id TEXT PRIMARY KEY,
                torrent_hash TEXT NOT NULL,
                file_path TEXT NOT NULL,
                password_hash TEXT,
                expiry_at INTEGER,
                created_at TEXT NOT NULL DEFAULT '',
                access_count INTEGER NOT NULL DEFAULT 0,
                bandwidth_used_bytes INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                level TEXT NOT NULL DEFAULT 'info',
                message TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT ''
            );
            ",
        )?;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<Settings> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let mut settings = Settings::default();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "downloads_dir" => settings.downloads_dir = value,
                "watch_dir" => settings.watch_dir = value,
                "max_download_speed_kb" => settings.max_download_speed_kb = value.parse().unwrap_or(0),
                "max_upload_speed_kb" => settings.max_upload_speed_kb = value.parse().unwrap_or(0),
                "max_connections" => settings.max_connections = value.parse().unwrap_or(200),
                "max_connections_per_torrent" => settings.max_connections_per_torrent = value.parse().unwrap_or(50),
                "dht_enabled" => settings.dht_enabled = value == "true",
                "pex_enabled" => settings.pex_enabled = value == "true",
                "lsd_enabled" => settings.lsd_enabled = value == "true",
                "utp_enabled" => settings.utp_enabled = value == "true",
                "listen_port" => settings.listen_port = value.parse().unwrap_or(6881),
                "webui_enabled" => settings.webui_enabled = value == "true",
                "webui_port" => settings.webui_port = value.parse().unwrap_or(8080),
                "webui_username" => settings.webui_username = value,
                "webui_password_hash" => settings.webui_password_hash = value,
                "auto_extract" => settings.auto_extract = value == "true",
                "extract_path" => settings.extract_path = value,
                "extract_passwords" => settings.extract_passwords = value,
                "rss_poll_interval_secs" => settings.rss_poll_interval_secs = value.parse().unwrap_or(900),
                "jackett_url" => settings.jackett_url = value,
                "jackett_api_key" => settings.jackett_api_key = value,
                "accent_color" => settings.accent_color = value,
                "theme" => settings.theme = value,
                "sequential_default" => settings.sequential_default = value == "true",
                "ratio_limit" => settings.ratio_limit = value.parse().unwrap_or(0.0),
                "seeding_time_limit_mins" => settings.seeding_time_limit_mins = value.parse().unwrap_or(0),
                "notifications_enabled" => settings.notifications_enabled = value == "true",
                "blocklist_url" => settings.blocklist_url = value,
                _ => {}
            }
        }
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let pairs: Vec<(&str, String)> = vec![
            ("downloads_dir", settings.downloads_dir.clone()),
            ("watch_dir", settings.watch_dir.clone()),
            ("max_download_speed_kb", settings.max_download_speed_kb.to_string()),
            ("max_upload_speed_kb", settings.max_upload_speed_kb.to_string()),
            ("max_connections", settings.max_connections.to_string()),
            ("max_connections_per_torrent", settings.max_connections_per_torrent.to_string()),
            ("dht_enabled", settings.dht_enabled.to_string()),
            ("pex_enabled", settings.pex_enabled.to_string()),
            ("lsd_enabled", settings.lsd_enabled.to_string()),
            ("utp_enabled", settings.utp_enabled.to_string()),
            ("listen_port", settings.listen_port.to_string()),
            ("webui_enabled", settings.webui_enabled.to_string()),
            ("webui_port", settings.webui_port.to_string()),
            ("webui_username", settings.webui_username.clone()),
            ("webui_password_hash", settings.webui_password_hash.clone()),
            ("auto_extract", settings.auto_extract.to_string()),
            ("extract_path", settings.extract_path.clone()),
            ("extract_passwords", settings.extract_passwords.clone()),
            ("rss_poll_interval_secs", settings.rss_poll_interval_secs.to_string()),
            ("jackett_url", settings.jackett_url.clone()),
            ("jackett_api_key", settings.jackett_api_key.clone()),
            ("accent_color", settings.accent_color.clone()),
            ("theme", settings.theme.clone()),
            ("sequential_default", settings.sequential_default.to_string()),
            ("ratio_limit", settings.ratio_limit.to_string()),
            ("seeding_time_limit_mins", settings.seeding_time_limit_mins.to_string()),
            ("notifications_enabled", settings.notifications_enabled.to_string()),
            ("blocklist_url", settings.blocklist_url.clone()),
        ];
        for (key, value) in pairs {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                rusqlite::params![key, value],
            )?;
        }
        Ok(())
    }

    pub fn insert_torrent(&self, record: &TorrentRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO torrents (id, info_hash, name, magnet_uri, save_path, total_bytes, downloaded_bytes, uploaded_bytes, status, category, tags, sequential, download_speed_limit, upload_speed_limit, ratio_limit, seeding_time_limit_mins, added_at, completed_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
            rusqlite::params![
                record.id, record.info_hash, record.name, record.magnet_uri, record.save_path,
                record.total_bytes, record.downloaded_bytes, record.uploaded_bytes, record.status,
                record.category, record.tags, record.sequential, record.download_speed_limit,
                record.upload_speed_limit, record.ratio_limit, record.seeding_time_limit_mins,
                record.added_at, record.completed_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_torrents(&self) -> Result<Vec<TorrentRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, info_hash, name, magnet_uri, save_path, total_bytes, downloaded_bytes, uploaded_bytes, status, category, tags, sequential, download_speed_limit, upload_speed_limit, ratio_limit, seeding_time_limit_mins, added_at, completed_at FROM torrents",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TorrentRecord {
                id: row.get(0)?,
                info_hash: row.get(1)?,
                name: row.get(2)?,
                magnet_uri: row.get(3)?,
                save_path: row.get(4)?,
                total_bytes: row.get(5)?,
                downloaded_bytes: row.get(6)?,
                uploaded_bytes: row.get(7)?,
                status: row.get(8)?,
                category: row.get(9)?,
                tags: row.get(10)?,
                sequential: row.get(11)?,
                download_speed_limit: row.get(12)?,
                upload_speed_limit: row.get(13)?,
                ratio_limit: row.get(14)?,
                seeding_time_limit_mins: row.get(15)?,
                added_at: row.get(16)?,
                completed_at: row.get(17)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn update_torrent_status(&self, id: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE torrents SET status = ?1 WHERE id = ?2",
            rusqlite::params![status, id],
        )?;
        Ok(())
    }

    pub fn update_torrent_bytes(&self, id: &str, downloaded: i64, uploaded: i64, total: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE torrents SET downloaded_bytes = ?1, uploaded_bytes = ?2, total_bytes = ?3 WHERE id = ?4",
            rusqlite::params![downloaded, uploaded, total, id],
        )?;
        Ok(())
    }

    pub fn delete_torrent(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM torrents WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn get_torrent_by_id(&self, id: &str) -> Result<Option<TorrentRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, info_hash, name, magnet_uri, save_path, total_bytes, downloaded_bytes, uploaded_bytes, status, category, tags, sequential, download_speed_limit, upload_speed_limit, ratio_limit, seeding_time_limit_mins, added_at, completed_at FROM torrents WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(TorrentRecord {
                id: row.get(0)?,
                info_hash: row.get(1)?,
                name: row.get(2)?,
                magnet_uri: row.get(3)?,
                save_path: row.get(4)?,
                total_bytes: row.get(5)?,
                downloaded_bytes: row.get(6)?,
                uploaded_bytes: row.get(7)?,
                status: row.get(8)?,
                category: row.get(9)?,
                tags: row.get(10)?,
                sequential: row.get(11)?,
                download_speed_limit: row.get(12)?,
                upload_speed_limit: row.get(13)?,
                ratio_limit: row.get(14)?,
                seeding_time_limit_mins: row.get(15)?,
                added_at: row.get(16)?,
                completed_at: row.get(17)?,
            })
        })?;
        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            _ => Ok(None),
        }
    }

    pub fn update_torrent_category(&self, id: &str, category: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE torrents SET category = ?1 WHERE id = ?2",
            rusqlite::params![category, id],
        )?;
        Ok(())
    }

    pub fn update_torrent_speed_limits(&self, id: &str, down_kb: i64, up_kb: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE torrents SET download_speed_limit = ?1, upload_speed_limit = ?2 WHERE id = ?3",
            rusqlite::params![down_kb, up_kb, id],
        )?;
        Ok(())
    }

    pub fn update_torrent_sequential(&self, id: &str, sequential: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE torrents SET sequential = ?1 WHERE id = ?2",
            rusqlite::params![sequential, id],
        )?;
        Ok(())
    }

    pub fn insert_extraction(&self, record: &ExtractionRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO extraction_queue (id, torrent_id, archive_path, output_dir, status, progress, password, error_message, started_at, completed_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            rusqlite::params![
                record.id, record.torrent_id, record.archive_path, record.output_dir,
                record.status, record.progress, record.password, record.error_message,
                record.started_at, record.completed_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_extractions(&self) -> Result<Vec<ExtractionRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, torrent_id, archive_path, output_dir, status, progress, password, error_message, started_at, completed_at FROM extraction_queue ORDER BY started_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ExtractionRecord {
                id: row.get(0)?,
                torrent_id: row.get(1)?,
                archive_path: row.get(2)?,
                output_dir: row.get(3)?,
                status: row.get(4)?,
                progress: row.get(5)?,
                password: row.get(6)?,
                error_message: row.get(7)?,
                started_at: row.get(8)?,
                completed_at: row.get(9)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn update_extraction_status(&self, id: &str, status: &str, progress: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE extraction_queue SET status = ?1, progress = ?2 WHERE id = ?3",
            rusqlite::params![status, progress, id],
        )?;
        Ok(())
    }

    pub fn update_extraction_password(&self, id: &str, password: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE extraction_queue SET password = ?1, status = 'queued' WHERE id = ?2",
            rusqlite::params![password, id],
        )?;
        Ok(())
    }

    pub fn update_extraction_error(&self, id: &str, error: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE extraction_queue SET status = 'error', error_message = ?1 WHERE id = ?2",
            rusqlite::params![error, id],
        )?;
        Ok(())
    }

    pub fn update_extraction_complete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE extraction_queue SET status = 'completed', progress = 100.0, completed_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id],
        )?;
        Ok(())
    }

    pub fn get_shared_links(&self) -> Result<Vec<SharedLinkRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, torrent_hash, file_path, password_hash, expiry_at, created_at, access_count, bandwidth_used_bytes FROM shared_links",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SharedLinkRecord {
                id: row.get(0)?,
                torrent_hash: row.get(1)?,
                file_path: row.get(2)?,
                password_hash: row.get(3)?,
                expiry_at: row.get(4)?,
                created_at: row.get(5)?,
                access_count: row.get(6)?,
                bandwidth_used_bytes: row.get(7)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn insert_shared_link(&self, record: &SharedLinkRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO shared_links (id, torrent_hash, file_path, password_hash, expiry_at, created_at, access_count, bandwidth_used_bytes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            rusqlite::params![
                record.id, record.torrent_hash, record.file_path, record.password_hash,
                record.expiry_at, record.created_at, record.access_count, record.bandwidth_used_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn save_shared_link(&self, record: &SharedLinkRecord) -> Result<()> {
        self.insert_shared_link(record)
    }

    pub fn delete_shared_link(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM shared_links WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn insert_rss_feed(&self, record: &RssFeedRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO rss_feeds (id, name, url, last_polled_at, last_etag) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![record.id, record.name, record.url, record.last_polled_at, record.last_etag],
        )?;
        Ok(())
    }

    pub fn get_rss_feeds(&self) -> Result<Vec<RssFeedRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, url, last_polled_at, last_etag FROM rss_feeds")?;
        let rows = stmt.query_map([], |row| {
            Ok(RssFeedRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                last_polled_at: row.get(3)?,
                last_etag: row.get(4)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn delete_rss_feed(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM rss_feeds WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn update_rss_feed_polled(&self, id: &str, etag: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE rss_feeds SET last_polled_at = ?1, last_etag = ?2 WHERE id = ?3",
            rusqlite::params![now, etag, id],
        )?;
        Ok(())
    }

    pub fn insert_rss_rule(&self, record: &RssRuleRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO rss_rules (id, name, pattern, feed_id, category, save_path, last_matched_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![record.id, record.name, record.pattern, record.feed_id, record.category, record.save_path, record.last_matched_at],
        )?;
        Ok(())
    }

    pub fn get_rss_rules(&self) -> Result<Vec<RssRuleRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, pattern, feed_id, category, save_path, last_matched_at FROM rss_rules")?;
        let rows = stmt.query_map([], |row| {
            Ok(RssRuleRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                pattern: row.get(2)?,
                feed_id: row.get(3)?,
                category: row.get(4)?,
                save_path: row.get(5)?,
                last_matched_at: row.get(6)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn delete_rss_rule(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM rss_rules WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn insert_log(&self, level: &str, message: &str, source: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO logs (level, message, source, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![level, message, source, now],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_creation_and_settings() {
        let db = Db::new(":memory:").unwrap();
        let settings = db.get_settings().unwrap();
        assert_eq!(settings.webui_port, 8080);
        assert_eq!(settings.accent_color, "blue");

        let mut new_settings = settings.clone();
        new_settings.webui_port = 9090;
        new_settings.accent_color = "purple".to_string();
        db.save_settings(&new_settings).unwrap();

        let loaded = db.get_settings().unwrap();
        assert_eq!(loaded.webui_port, 9090);
        assert_eq!(loaded.accent_color, "purple");
    }

    #[test]
    fn test_torrent_crud() {
        let db = Db::new(":memory:").unwrap();
        let record = TorrentRecord {
            id: "test-1".to_string(),
            info_hash: "abc123".to_string(),
            name: "Test Torrent".to_string(),
            magnet_uri: "magnet:?xt=urn:btih:abc123".to_string(),
            save_path: "/tmp/downloads".to_string(),
            total_bytes: 1000000,
            downloaded_bytes: 0,
            uploaded_bytes: 0,
            status: "queued".to_string(),
            category: "movies".to_string(),
            tags: "hd,action".to_string(),
            sequential: false,
            download_speed_limit: 0,
            upload_speed_limit: 0,
            ratio_limit: 0.0,
            seeding_time_limit_mins: 0,
            added_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        };

        db.insert_torrent(&record).unwrap();
        let torrents = db.get_torrents().unwrap();
        assert_eq!(torrents.len(), 1);
        assert_eq!(torrents[0].name, "Test Torrent");

        db.update_torrent_status("test-1", "downloading").unwrap();
        let t = db.get_torrent_by_id("test-1").unwrap().unwrap();
        assert_eq!(t.status, "downloading");

        db.update_torrent_category("test-1", "music").unwrap();
        let t = db.get_torrent_by_id("test-1").unwrap().unwrap();
        assert_eq!(t.category, "music");

        db.delete_torrent("test-1").unwrap();
        let torrents = db.get_torrents().unwrap();
        assert_eq!(torrents.len(), 0);
    }

    #[test]
    fn test_extraction_crud() {
        let db = Db::new(":memory:").unwrap();
        let record = ExtractionRecord {
            id: "ext-1".to_string(),
            torrent_id: "test-1".to_string(),
            archive_path: "/tmp/test.zip".to_string(),
            output_dir: "/tmp/extracted".to_string(),
            status: "pending".to_string(),
            progress: 0.0,
            password: None,
            error_message: None,
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        };

        db.insert_extraction(&record).unwrap();
        let extractions = db.get_extractions().unwrap();
        assert_eq!(extractions.len(), 1);

        db.update_extraction_status("ext-1", "processing", 50.0).unwrap();
        let ext = db.get_extractions().unwrap();
        assert_eq!(ext[0].status, "processing");
        assert!((ext[0].progress - 50.0).abs() < 0.01);

        db.update_extraction_complete("ext-1").unwrap();
        let ext = db.get_extractions().unwrap();
        assert_eq!(ext[0].status, "completed");
    }

    #[test]
    fn test_shared_links_crud() {
        let db = Db::new(":memory:").unwrap();
        let record = SharedLinkRecord {
            id: "share-1".to_string(),
            torrent_hash: "abc123".to_string(),
            file_path: "movie.mp4".to_string(),
            password_hash: None,
            expiry_at: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            access_count: 0,
            bandwidth_used_bytes: 0,
        };

        db.insert_shared_link(&record).unwrap();
        let links = db.get_shared_links().unwrap();
        assert_eq!(links.len(), 1);

        db.delete_shared_link("share-1").unwrap();
        let links = db.get_shared_links().unwrap();
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_rss_feeds_and_rules() {
        let db = Db::new(":memory:").unwrap();

        let feed = RssFeedRecord {
            id: "feed-1".to_string(),
            name: "Test Feed".to_string(),
            url: "https://example.com/rss".to_string(),
            last_polled_at: None,
            last_etag: None,
        };
        db.insert_rss_feed(&feed).unwrap();
        let feeds = db.get_rss_feeds().unwrap();
        assert_eq!(feeds.len(), 1);

        let rule = RssRuleRecord {
            id: "rule-1".to_string(),
            name: "Test Rule".to_string(),
            pattern: ".*1080p.*".to_string(),
            feed_id: Some("feed-1".to_string()),
            category: "movies".to_string(),
            save_path: "/tmp/movies".to_string(),
            last_matched_at: None,
        };
        db.insert_rss_rule(&rule).unwrap();
        let rules = db.get_rss_rules().unwrap();
        assert_eq!(rules.len(), 1);

        db.delete_rss_rule("rule-1").unwrap();
        let rules = db.get_rss_rules().unwrap();
        assert_eq!(rules.len(), 0);

        db.delete_rss_feed("feed-1").unwrap();
        let feeds = db.get_rss_feeds().unwrap();
        assert_eq!(feeds.len(), 0);
    }
}
