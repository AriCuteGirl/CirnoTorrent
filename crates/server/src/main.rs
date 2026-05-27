use cirnotorrent_core::Engine;
use clap::Parser;
use rust_embed::Embed;
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

#[derive(Embed)]
#[folder = "../../frontend/dist/"]
struct FrontendAssets;

#[derive(Parser)]
#[command(name = "cirnotorrent-server", about = "Cirnotorrent headless server")]
struct Cli {
    #[arg(long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "./downloads")]
    download_path: String,

    #[arg(long, default_value = "admin")]
    username: String,

    #[arg(long, default_value = "")]
    password: String,

    #[arg(long, default_value = "cirnotorrent.db")]
    db_path: String,

    #[arg(long, default_value = "0")]
    max_down_speed: i64,

    #[arg(long, default_value = "0")]
    max_up_speed: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    info!("Starting Cirnotorrent server on port {}", cli.port);
    info!("Download path: {}", cli.download_path);
    info!("Database: {}", cli.db_path);

    std::fs::create_dir_all(&cli.download_path)?;

    let engine = Engine::new(&cli.db_path)?;

    let mut settings = engine.db.get_settings().unwrap_or_default();
    settings.downloads_dir = cli.download_path.clone();
    settings.webui_port = cli.port as i32;
    settings.webui_username = cli.username.clone();
    settings.max_download_speed_kb = cli.max_down_speed;
    settings.max_upload_speed_kb = cli.max_up_speed;
    settings.webui_enabled = true;

    if !cli.password.is_empty() {
        settings.webui_password_hash = bcrypt::hash(&cli.password, 10)?;
    }

    engine.db.save_settings(&settings)?;

    let jwt_secret = uuid::Uuid::new_v4().to_string();
    let app_state = engine.to_app_state(jwt_secret);
    let api_router = cirnotorrent_core::api::create_router(app_state);

    let app = axum::Router::new()
        .merge(api_router)
        .fallback(serve_frontend);

    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down gracefully");
    Ok(())
}

async fn serve_frontend(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match FrontendAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                axum::http::StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, mime.as_ref().to_string())],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => {
            match FrontendAssets::get("index.html") {
                Some(content) => (
                    axum::http::StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, "text/html".to_string())],
                    content.data.to_vec(),
                )
                    .into_response(),
                None => (
                    axum::http::StatusCode::NOT_FOUND,
                    "Frontend not built or not found",
                )
                    .into_response(),
            }
        }
    }
}

use axum::response::IntoResponse;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { info!("Received Ctrl+C, shutting down"); }
        _ = terminate => { info!("Received SIGTERM, shutting down"); }
    }
}
