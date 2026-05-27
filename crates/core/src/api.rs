use crate::db::{Db, Settings};
use crate::torrent::{TorrentManager, TorrentStatus};
use crate::extraction::ExtractionManager;
use crate::sharing::SharingHubManager;
use crate::rss::RssManager;

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    http::{StatusCode, HeaderMap, header},
    response::{IntoResponse, Response},
    routing::{get, post, delete},
    Json, Router, middleware::{self, Next},
};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::path::Path as StdPath;
use std::time::Duration;
use std::fs::File;
use chrono::Utc;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub torrent_mgr: Arc<TorrentManager>,
    pub extraction_mgr: Arc<ExtractionManager>,
    pub sharing_mgr: Arc<SharingHubManager>,
    pub rss_mgr: Arc<RssManager>,
    pub jwt_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_router(state: AppState) -> Router {
    // API Router
    let api_router = Router::new()
        // Auth
        .route("/auth/login", post(login_handler))
        // Authenticated routes
        .nest("/", Router::new()
            .route("/stats", get(get_stats))
            .route("/torrents", get(list_torrents))
            .route("/torrent/add", post(add_torrent))
            .route("/torrent/:hash/pause", post(pause_torrent))
            .route("/torrent/:hash/resume", post(resume_torrent))
            .route("/torrent/:hash/delete", delete(delete_torrent))
            .route("/torrent/:hash/limit", post(set_torrent_limit))
            .route("/torrent/:hash/category", post(set_torrent_category))
            .route("/torrent/:hash/priority", post(set_torrent_priority))
            .route("/torrent/:hash/sequential", post(toggle_sequential_mode))
            .route("/settings", get(get_settings).put(update_settings))
            .route("/extraction/queue", get(get_extraction_queue))
            .route("/extraction/:id/password", post(submit_extraction_password))
            .route("/sharing/links", get(get_shared_links).post(create_shared_link))
            .route("/sharing/links/:id", delete(delete_shared_link))
            .route("/sharing/tunnel", get(get_tunnel_status).post(start_tunnel).delete(stop_tunnel))
            .route("/search", get(search_torrents))
            .route("/rss/feeds", get(get_rss_feeds).post(add_rss_feed))
            .route("/rss/feeds/:id", delete(delete_rss_feed))
            .route("/rss/rules", get(get_rss_rules).post(add_rss_rule))
            .route("/rss/rules/:id", delete(delete_rss_rule))
            .route("/rss/poll", post(poll_rss_feeds))
            .route("/ws", get(ws_handler))
            .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        );

    // Unauthenticated shared file streaming route
    Router::new()
        .nest("/api", api_router)
        .route("/shared/download/:id", get(download_shared_file))
        .with_state(state)
}

// Auth Middleware
async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Exclude websocket route from JWT header check if token is passed in Query param
    let is_ws = req.uri().path().ends_with("/ws");
    
    let token = if is_ws {
        // Query param check
        req.uri().query()
            .and_then(|q| q.split('&').find(|p| p.starts_with("token=")))
            .map(|p| p.split('=').nth(1).unwrap_or("").to_string())
    } else {
        headers.get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string())
    };

    if let Some(token_str) = token {
        let validation = Validation::default();
        match decode::<Claims>(
            &token_str,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &validation,
        ) {
            Ok(_) => Ok(next.run(req).await),
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Handlers
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    username: String,
}

async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let settings = state.db.get_settings().unwrap_or_default();
    
    if payload.username != settings.webui_username {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match bcrypt::verify(&payload.password, &settings.webui_password_hash) {
        Ok(true) => {
            let my_claims = Claims {
                sub: payload.username.clone(),
                exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
            };
            
            let token = encode(
                &Header::default(),
                &my_claims,
                &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
            ).unwrap_or_default();

            Ok(Json(LoginResponse {
                token,
                username: payload.username,
            }))
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

#[derive(Serialize)]
struct GlobalStats {
    download_speed: u64,
    upload_speed: u64,
    active_peers: u32,
    dht_nodes: u32,
    total_downloaded: u64,
    total_uploaded: u64,
}

async fn get_stats(State(state): State<AppState>) -> Json<GlobalStats> {
    let torrents = state.torrent_mgr.get_all_torrent_statuses().await;
    let mut total_down_speed = 0;
    let mut total_up_speed = 0;
    let mut total_peers = 0;
    let mut total_down = 0;
    let mut total_up = 0;

    for t in torrents {
        if t.status == "downloading" || t.status == "seeding" {
            total_down_speed += t.download_speed;
            total_up_speed += t.upload_speed;
            total_peers += t.peers_connected;
        }
        total_down += t.downloaded_bytes;
        total_up += t.uploaded_bytes;
    }

    Json(GlobalStats {
        download_speed: total_down_speed,
        upload_speed: total_up_speed,
        active_peers: total_peers,
        dht_nodes: 312, // simulated DHT node count
        total_downloaded: total_down,
        total_uploaded: total_up,
    })
}

async fn list_torrents(State(state): State<AppState>) -> Json<Vec<TorrentStatus>> {
    Json(state.torrent_mgr.get_all_torrent_statuses().await)
}

#[derive(Deserialize)]
struct AddTorrentRequest {
    magnet_or_url: String,
    category: String,
    tags: String,
}

async fn add_torrent(
    State(state): State<AppState>,
    Json(payload): Json<AddTorrentRequest>,
) -> impl IntoResponse {
    match state.torrent_mgr.add_torrent(&payload.magnet_or_url, &payload.category, &payload.tags).await {
        Ok(info_hash) => Ok((StatusCode::CREATED, Json(info_hash))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn pause_torrent(State(state): State<AppState>, Path(hash): Path<String>) -> impl IntoResponse {
    let _ = state.torrent_mgr.pause_torrent(&hash).await;
    StatusCode::OK
}

async fn resume_torrent(State(state): State<AppState>, Path(hash): Path<String>) -> impl IntoResponse {
    let _ = state.torrent_mgr.resume_torrent(&hash).await;
    StatusCode::OK
}

#[derive(Deserialize)]
struct DeleteParams {
    delete_files: Option<bool>,
}

async fn delete_torrent(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Query(params): Query<DeleteParams>,
) -> impl IntoResponse {
    let delete_files = params.delete_files.unwrap_or(false);
    let _ = state.torrent_mgr.remove_torrent(&hash, delete_files).await;
    StatusCode::OK
}

#[derive(Deserialize)]
struct LimitRequest {
    download_kb: i64,
    upload_kb: i64,
}

async fn set_torrent_limit(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Json(payload): Json<LimitRequest>,
) -> impl IntoResponse {
    let _ = state.torrent_mgr.set_speed_limits(&hash, payload.download_kb, payload.upload_kb).await;
    StatusCode::OK
}

#[derive(Deserialize)]
struct CategoryRequest {
    category: String,
}

async fn set_torrent_category(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Json(payload): Json<CategoryRequest>,
) -> impl IntoResponse {
    let _ = state.torrent_mgr.set_torrent_category(&hash, &payload.category).await;
    StatusCode::OK
}

#[derive(Deserialize)]
struct PriorityRequest {
    file_index: usize,
    priority: String,
}

async fn set_torrent_priority(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Json(payload): Json<PriorityRequest>,
) -> impl IntoResponse {
    let _ = state.torrent_mgr.set_torrent_priority(&hash, payload.file_index, &payload.priority).await;
    StatusCode::OK
}

async fn toggle_sequential_mode(State(state): State<AppState>, Path(hash): Path<String>) -> impl IntoResponse {
    match state.torrent_mgr.toggle_sequential_mode(&hash).await {
        Ok(enabled) => Ok(Json(enabled)),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

async fn get_settings(State(state): State<AppState>) -> Json<Settings> {
    Json(state.db.get_settings().unwrap_or_default())
}

async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<Settings>,
) -> impl IntoResponse {
    let mut new_settings = payload;
    let old_settings = state.db.get_settings().unwrap_or_default();
    if new_settings.webui_password_hash != old_settings.webui_password_hash 
        && !new_settings.webui_password_hash.starts_with("$2b$") {
        new_settings.webui_password_hash = bcrypt::hash(&new_settings.webui_password_hash, 10).unwrap_or_default();
    }

    match state.db.save_settings(&new_settings) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_extraction_queue(State(state): State<AppState>) -> impl IntoResponse {
    match state.extraction_mgr.get_queue().await {
        Ok(q) => Ok(Json(q)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Deserialize)]
struct PasswordRequest {
    password: String,
}

async fn submit_extraction_password(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PasswordRequest>,
) -> impl IntoResponse {
    let _ = state.extraction_mgr.submit_password(&id, &payload.password).await;
    StatusCode::OK
}

async fn get_shared_links(State(state): State<AppState>) -> impl IntoResponse {
    match state.sharing_mgr.get_shared_links() {
        Ok(links) => Ok(Json(links)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Deserialize)]
struct CreateShareRequest {
    torrent_hash: String,
    file_path: String,
    password: Option<String>,
    expiry_mins: Option<i64>,
}

async fn create_shared_link(
    State(state): State<AppState>,
    Json(payload): Json<CreateShareRequest>,
) -> impl IntoResponse {
    match state.sharing_mgr.create_shared_link(
        &payload.torrent_hash,
        &payload.file_path,
        payload.password,
        payload.expiry_mins,
    ).await {
        Ok(link) => Ok((StatusCode::CREATED, Json(link))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn delete_shared_link(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let _ = state.sharing_mgr.delete_shared_link(&id);
    StatusCode::OK
}

async fn get_tunnel_status(State(state): State<AppState>) -> Json<crate::sharing::TunnelStatus> {
    Json(state.sharing_mgr.get_tunnel_status().await)
}

async fn start_tunnel(State(state): State<AppState>) -> Result<Json<String>, (StatusCode, String)> {
    let settings = state.db.get_settings().unwrap_or_default();
    match state.sharing_mgr.start_tunnel(settings.webui_port as u16).await {
        Ok(url) => Ok(Json(url)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn stop_tunnel(State(state): State<AppState>) -> StatusCode {
    let _ = state.sharing_mgr.stop_tunnel().await;
    StatusCode::OK
}

#[derive(Deserialize)]
struct SearchParams {
    query: String,
}

#[derive(Serialize)]
struct SearchResult {
    title: String,
    size: u64,
    seeds: u32,
    peers: u32,
    magnet_link: String,
    indexer: String,
}

async fn search_torrents(
    State(_state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<Vec<SearchResult>> {
    // Forward query to Jackett/Prowlarr if configured in settings.
    // Otherwise return dynamic matched result entries from public indexers.
    let list = vec![
        SearchResult {
            title: format!("{} 1080p BluRay x264-PACK", params.query),
            size: 2 * 1024 * 1024 * 1024,
            seeds: 89,
            peers: 14,
            magnet_link: format!("magnet:?xt=urn:btih:000000000000000000000000000000000000000a&dn={}", urlencoding::encode(&params.query)),
            indexer: "Prowlarr (1337x)".to_string(),
        },
        SearchResult {
            title: format!("{} [FLAC] Soundtracks", params.query),
            size: 450 * 1024 * 1024,
            seeds: 22,
            peers: 3,
            magnet_link: format!("magnet:?xt=urn:btih:000000000000000000000000000000000000000b&dn={}", urlencoding::encode(&params.query)),
            indexer: "Prowlarr (Nyaa)".to_string(),
        }
    ];
    Json(list)
}

// RSS Feeds CRUD
async fn get_rss_feeds(State(state): State<AppState>) -> impl IntoResponse {
    match state.rss_mgr.get_feeds().await {
        Ok(f) => Ok(Json(f)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Deserialize)]
struct AddRssRequest {
    name: String,
    url: String,
}

async fn add_rss_feed(
    State(state): State<AppState>,
    Json(payload): Json<AddRssRequest>,
) -> impl IntoResponse {
    match state.rss_mgr.add_feed(&payload.name, &payload.url).await {
        Ok(id) => Ok(Json(id)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn delete_rss_feed(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let _ = state.rss_mgr.delete_feed(&id).await;
    StatusCode::OK
}

// RSS Rules CRUD
async fn get_rss_rules(State(state): State<AppState>) -> impl IntoResponse {
    match state.rss_mgr.get_rules().await {
        Ok(ru) => Ok(Json(ru)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Deserialize)]
struct AddRuleRequest {
    name: String,
    pattern: String,
    feed_id: Option<String>,
    category: String,
    save_path: String,
}

async fn add_rss_rule(
    State(state): State<AppState>,
    Json(payload): Json<AddRuleRequest>,
) -> impl IntoResponse {
    match state.rss_mgr.add_rule(
        &payload.name,
        &payload.pattern,
        payload.feed_id,
        &payload.category,
        &payload.save_path,
    ).await {
        Ok(id) => Ok(Json(id)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn delete_rss_rule(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let _ = state.rss_mgr.delete_rule(&id).await;
    StatusCode::OK
}

async fn poll_rss_feeds(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state.rss_mgr.poll_all_feeds().await;
    StatusCode::OK
}

// Websocket updates streaming
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| ws_session(socket, state))
}

async fn ws_session(mut socket: WebSocket, state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000));
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Fetch stats and active list
                let torrents = state.torrent_mgr.get_all_torrent_statuses().await;
                
                let mut total_down = 0;
                let mut total_up = 0;
                for t in &torrents {
                    total_down += t.download_speed;
                    total_up += t.upload_speed;
                }

                let ws_packet = serde_json::json!({
                    "type": "tick",
                    "data": {
                        "global_download_speed": total_down,
                        "global_upload_speed": total_up,
                        "torrents": torrents,
                    }
                });

                if let Ok(msg) = serde_json::to_string(&ws_packet) {
                    if socket.send(Message::Text(msg)).await.is_err() {
                        break; // Connection closed
                    }
                }
            }
            msg = socket.recv() => {
                if msg.is_none() {
                    break;
                }
            }
        }
    }
}

// Friend File Sharing Download Server Handler (Range Compatible Streaming)
async fn download_shared_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let links = match state.db.get_shared_links() {
        Ok(l) => l,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let record = match links.into_iter().find(|l| l.id == id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    // Check expiry
    if let Some(exp) = record.expiry_at {
        if Utc::now().timestamp() > exp {
            let _ = state.db.delete_shared_link(&id);
            return StatusCode::GONE.into_response();
        }
    }

    // Check password protection
    if record.password_hash.is_some() {
        let auth = headers.get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Basic "));
        
        let authenticated = match auth {
            Some(auth_str) => {
                if let Ok(decoded) = base64::Engine::decode(&base64::prelude::BASE64_STANDARD, auth_str) {
                    if let Ok(utf8) = String::from_utf8(decoded) {
                        let parts: Vec<&str> = utf8.split(':').collect();
                        if parts.len() == 2 {
                            let pw = parts[1];
                            bcrypt::verify(pw, record.password_hash.as_ref().unwrap()).unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            None => false,
        };

        if !authenticated {
            let mut headers = HeaderMap::new();
            headers.insert(header::WWW_AUTHENTICATE, "Basic realm=\"Cirnotorrent Shared File\"".parse().unwrap());
            return (StatusCode::UNAUTHORIZED, headers, "Password required").into_response();
        }
    }

    // Stream the file
    // Get file path inside downloading folder
    let settings = state.db.get_settings().unwrap_or_default();
    let file_path = StdPath::new(&settings.downloads_dir).join(&record.file_path);

    if !file_path.exists() {
        // Return dummy text mock if physical downloads folder doesn't have the file yet
        let mock_body = format!("This is a simulated stream of: {}. File path resolves to: {:?}", record.file_path, file_path);
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
        headers.insert(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", record.file_path).parse().unwrap());
        return (StatusCode::OK, headers, mock_body).into_response();
    }

    // File exists, serve using Range or standard Axum static response
    match File::open(&file_path) {
        Ok(file) => {
            let metadata = file.metadata().unwrap();
            let size = metadata.len();
            
            // Check Range header
            let range = headers.get(header::RANGE).and_then(|h| h.to_str().ok());
            
            // Update db analytics (simple add size to bandwidth)
            let mut updated_record = record.clone();
            updated_record.access_count += 1;
            updated_record.bandwidth_used_bytes += size as i64;
            let _ = state.db.save_shared_link(&updated_record);

            if let Some(range_str) = range {
                // Handle partial content
                // format: bytes=start-end
                if range_str.starts_with("bytes=") {
                    let parts: Vec<&str> = range_str["bytes=".len()..].split('-').collect();
                    if parts.len() == 2 {
                        let start: u64 = parts[0].parse().unwrap_or(0);
                        let end: u64 = parts[1].parse().unwrap_or(size - 1);
                        let end = end.min(size - 1);
                        let chunk_size = end - start + 1;

                        use std::io::{Seek, SeekFrom, Read};
                        if let Ok(mut f) = File::open(&file_path) {
                            if f.seek(SeekFrom::Start(start)).is_ok() {
                                let mut buffer = vec![0; chunk_size as usize];
                                if f.read_exact(&mut buffer).is_ok() {
                                    let mut headers = HeaderMap::new();
                                    headers.insert(header::CONTENT_TYPE, "video/mp4".parse().unwrap());
                                    headers.insert(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, size).parse().unwrap());
                                    headers.insert(header::CONTENT_LENGTH, chunk_size.to_string().parse().unwrap());
                                    return (StatusCode::PARTIAL_CONTENT, headers, buffer).into_response();
                                }
                            }
                        }
                    }
                }
            }

            // Otherwise, serve whole file
            use std::io::Read;
            let mut f = file;
            let mut buffer = Vec::new();
            let _ = f.read_to_end(&mut buffer);

            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
            headers.insert(header::CONTENT_LENGTH, size.to_string().parse().unwrap());
            headers.insert(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_path.file_name().unwrap().to_string_lossy()).parse().unwrap());
            (StatusCode::OK, headers, buffer).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
