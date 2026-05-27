// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cirnotorrent_core::Engine;
use cirnotorrent_core::db::{Settings, ExtractionRecord, SharedLinkRecord, RssFeedRecord, RssRuleRecord};
use cirnotorrent_core::torrent::TorrentStatus;

use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::time::Duration;

// Tauri Commands mapping directly to crates/core Engine APIs
#[tauri::command]
async fn add_torrent(
    engine: State<'_, Arc<Engine>>,
    magnet_or_url: String,
    category: String,
    tags: String,
) -> Result<String, String> {
    engine.torrent_mgr.add_torrent(&magnet_or_url, &category, &tags).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_torrent_file(
    engine: State<'_, Arc<Engine>>,
    file_path: String,
    category: String,
    tags: String,
) -> Result<String, String> {
    let data = std::fs::read(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    engine.torrent_mgr.add_torrent_from_bytes(data, &category, &tags).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_torrent_file_bytes(
    engine: State<'_, Arc<Engine>>,
    data: Vec<u8>,
    category: String,
    tags: String,
) -> Result<String, String> {
    engine.torrent_mgr.add_torrent_from_bytes(data, &category, &tags).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_torrents(engine: State<'_, Arc<Engine>>) -> Result<Vec<TorrentStatus>, String> {
    Ok(engine.torrent_mgr.get_all_torrent_statuses().await)
}

#[tauri::command]
async fn pause_torrent(engine: State<'_, Arc<Engine>>, hash: String) -> Result<(), String> {
    engine.torrent_mgr.pause_torrent(&hash).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn resume_torrent(engine: State<'_, Arc<Engine>>, hash: String) -> Result<(), String> {
    engine.torrent_mgr.resume_torrent(&hash).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_torrent(engine: State<'_, Arc<Engine>>, hash: String, delete_files: bool) -> Result<(), String> {
    engine.torrent_mgr.remove_torrent(&hash, delete_files).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_torrent_limit(engine: State<'_, Arc<Engine>>, hash: String, download_kb: i64, upload_kb: i64) -> Result<(), String> {
    engine.torrent_mgr.set_speed_limits(&hash, download_kb, upload_kb).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_torrent_category(engine: State<'_, Arc<Engine>>, hash: String, category: String) -> Result<(), String> {
    engine.torrent_mgr.set_torrent_category(&hash, &category).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_torrent_priority(engine: State<'_, Arc<Engine>>, hash: String, file_index: usize, priority: String) -> Result<(), String> {
    engine.torrent_mgr.set_torrent_priority(&hash, file_index, &priority).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_sequential_mode(engine: State<'_, Arc<Engine>>, hash: String) -> Result<bool, String> {
    engine.torrent_mgr.toggle_sequential_mode(&hash).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_settings(engine: State<'_, Arc<Engine>>) -> Result<Settings, String> {
    engine.db.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_settings(engine: State<'_, Arc<Engine>>, settings: Settings) -> Result<(), String> {
    let mut new_settings = settings;
    // Password checking
    let old_settings = engine.db.get_settings().map_err(|e| e.to_string())?;
    if new_settings.webui_password_hash != old_settings.webui_password_hash 
        && !new_settings.webui_password_hash.starts_with("$2b$") {
        new_settings.webui_password_hash = bcrypt::hash(&new_settings.webui_password_hash, 10).map_err(|e| e.to_string())?;
    }
    
    engine.db.save_settings(&new_settings).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_extraction_queue(engine: State<'_, Arc<Engine>>) -> Result<Vec<ExtractionRecord>, String> {
    engine.extraction_mgr.get_queue().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn submit_extraction_password(engine: State<'_, Arc<Engine>>, id: String, password: String) -> Result<(), String> {
    engine.extraction_mgr.submit_password(&id, &password).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_shared_links(engine: State<'_, Arc<Engine>>) -> Result<Vec<SharedLinkRecord>, String> {
    engine.sharing_mgr.get_shared_links().map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_shared_link(
    engine: State<'_, Arc<Engine>>, 
    torrent_hash: String, 
    file_path: String, 
    password: Option<String>, 
    expiry_mins: Option<i64>
) -> Result<SharedLinkRecord, String> {
    engine.sharing_mgr.create_shared_link(&torrent_hash, &file_path, password, expiry_mins).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_shared_link(engine: State<'_, Arc<Engine>>, id: String) -> Result<(), String> {
    engine.sharing_mgr.delete_shared_link(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tunnel_status(engine: State<'_, Arc<Engine>>) -> Result<cirnotorrent_core::sharing::TunnelStatus, String> {
    Ok(engine.sharing_mgr.get_tunnel_status().await)
}

#[tauri::command]
async fn start_tunnel(engine: State<'_, Arc<Engine>>) -> Result<String, String> {
    let settings = engine.db.get_settings().map_err(|e| e.to_string())?;
    engine.sharing_mgr.start_tunnel(settings.webui_port as u16).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_tunnel(engine: State<'_, Arc<Engine>>) -> Result<(), String> {
    engine.sharing_mgr.stop_tunnel().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_rss_feeds(engine: State<'_, Arc<Engine>>) -> Result<Vec<RssFeedRecord>, String> {
    engine.rss_mgr.get_feeds().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_rss_feed(engine: State<'_, Arc<Engine>>, name: String, url: String) -> Result<String, String> {
    engine.rss_mgr.add_feed(&name, &url).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_rss_feed(engine: State<'_, Arc<Engine>>, id: String) -> Result<(), String> {
    engine.rss_mgr.delete_feed(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_rss_rules(engine: State<'_, Arc<Engine>>) -> Result<Vec<RssRuleRecord>, String> {
    engine.rss_mgr.get_rules().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_rss_rule(
    engine: State<'_, Arc<Engine>>, 
    name: String, 
    pattern: String, 
    feed_id: Option<String>, 
    category: String, 
    save_path: String
) -> Result<String, String> {
    engine.rss_mgr.add_rule(&name, &pattern, feed_id, &category, &save_path).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_rss_rule(engine: State<'_, Arc<Engine>>, id: String) -> Result<(), String> {
    engine.rss_mgr.delete_rule(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn poll_rss_feeds(engine: State<'_, Arc<Engine>>) -> Result<(), String> {
    engine.rss_mgr.poll_all_feeds().await.map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_torrents(
    query: String
) -> Result<Vec<serde_json::Value>, String> {
    // Return dummy search results in desktop mode
    let list = vec![
        serde_json::json!({
            "title": format!("{} 1080p BluRay x264-PACK", query),
            "size": 2147483648u64,
            "seeds": 89,
            "peers": 14,
            "magnet_link": format!("magnet:?xt=urn:btih:000000000000000000000000000000000000000a&dn={}", urlencoding::encode(&query)),
            "indexer": "Prowlarr (1337x)",
        }),
        serde_json::json!({
            "title": format!("{} [FLAC] Soundtracks", query),
            "size": 471859200u64,
            "seeds": 22,
            "peers": 3,
            "magnet_link": format!("magnet:?xt=urn:btih:000000000000000000000000000000000000000b&dn={}", urlencoding::encode(&query)),
            "indexer": "Prowlarr (Nyaa)",
        })
    ];
    Ok(list)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Locate or create app local database file
    let app_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
    let db_dir = app_dir.join("cirnotorrent");
    std::fs::create_dir_all(&db_dir)?;
    let db_path = db_dir.join("cirnotorrent.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    println!("Desktop mode initializing DB at: {}", db_path_str);
    let engine = Arc::new(Engine::new(&db_path_str)?);

    // If Web UI setting is enabled, start the Axum HTTP server in the background
    let engine_for_server = engine.clone();
    tauri::async_runtime::spawn(async move {
        if let Ok(settings) = engine_for_server.db.get_settings() {
            if settings.webui_enabled {
                println!("Web UI enabled on port {} inside Desktop App", settings.webui_port);
                let app_state = engine_for_server.to_app_state("desktop_secret_jwt_key_2026".to_string());
                let app = cirnotorrent_core::api::create_router(app_state);
                let addr = std::net::SocketAddr::from(([0, 0, 0, 0], settings.webui_port as u16));
                if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
                    let _ = axum::serve(listener, app).await;
                }
            }
        }
    });

    let engine_for_emit = engine.clone();

    tauri::Builder::default()
        .manage(engine)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            // Start background thread that emits ticking updates to front-end window
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                loop {
                    interval.tick().await;
                    
                    let torrents = engine_for_emit.torrent_mgr.get_all_torrent_statuses().await;
                    let mut total_down = 0;
                    let mut total_up = 0;
                    for t in &torrents {
                        total_down += t.download_speed;
                        total_up += t.upload_speed;
                    }

                    let payload = serde_json::json!({
                        "global_download_speed": total_down,
                        "global_upload_speed": total_up,
                        "torrents": torrents,
                    });

                    // Emit event using Emitter trait in Tauri v2
                    let _ = app_handle.emit("tick", payload);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_torrent,
            add_torrent_file,
            add_torrent_file_bytes,
            get_torrents,
            pause_torrent,
            resume_torrent,
            delete_torrent,
            set_torrent_limit,
            set_torrent_category,
            set_torrent_priority,
            toggle_sequential_mode,
            get_settings,
            update_settings,
            get_extraction_queue,
            submit_extraction_password,
            get_shared_links,
            create_shared_link,
            delete_shared_link,
            get_tunnel_status,
            start_tunnel,
            stop_tunnel,
            get_rss_feeds,
            add_rss_feed,
            delete_rss_feed,
            get_rss_rules,
            add_rss_rule,
            delete_rss_rule,
            poll_rss_feeds,
            search_torrents
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
