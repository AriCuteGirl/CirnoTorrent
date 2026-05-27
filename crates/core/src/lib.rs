pub mod api;
pub mod db;
pub mod extraction;
pub mod rss;
pub mod sharing;
pub mod torrent;

use std::sync::Arc;
use anyhow::Result;

pub struct Engine {
    pub db: db::Db,
    pub torrent_mgr: Arc<torrent::TorrentManager>,
    pub extraction_mgr: Arc<extraction::ExtractionManager>,
    pub sharing_mgr: Arc<sharing::SharingHubManager>,
    pub rss_mgr: Arc<rss::RssManager>,
}

impl Engine {
    pub fn new(db_path: &str) -> Result<Self> {
        let db = db::Db::new(db_path)?;
        let download_dir = db.get_settings().unwrap_or_default().downloads_dir;

        let torrent_mgr = Arc::new(torrent::TorrentManager::new(db.clone(), &download_dir)?);
        let extraction_mgr = Arc::new(extraction::ExtractionManager::new(db.clone())?);
        let sharing_mgr = Arc::new(sharing::SharingHubManager::new(db.clone())?);
        let rss_mgr = Arc::new(rss::RssManager::new(db.clone())?);

        Ok(Self {
            db,
            torrent_mgr,
            extraction_mgr,
            sharing_mgr,
            rss_mgr,
        })
    }

    pub fn to_app_state(&self, jwt_secret: String) -> api::AppState {
        api::AppState {
            db: self.db.clone(),
            torrent_mgr: self.torrent_mgr.clone(),
            extraction_mgr: self.extraction_mgr.clone(),
            sharing_mgr: self.sharing_mgr.clone(),
            rss_mgr: self.rss_mgr.clone(),
            jwt_secret,
        }
    }
}
