mod diff;
mod scgi;
mod sse;
mod xmlrpc;

use axum::{error_handling::HandleErrorLayer, BoxError};
use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use rust_embed::RustEmbed;
use serde::Deserialize;
use shared::{AppEvent, Torrent, TorrentActionRequest}; // shared crates imports
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, watch};
use tower::ServiceBuilder;
use tower_http::{
    compression::{CompressionLayer, CompressionLevel},
    cors::CorsLayer,
    trace::TraceLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub tx: Arc<watch::Sender<Vec<Torrent>>>,
    pub event_bus: broadcast::Sender<AppEvent>,
    pub scgi_socket_path: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to rTorrent SCGI socket
    #[arg(short, long, default_value = "/tmp/rtorrent.sock")]
    socket: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct Asset;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return StatusCode::NOT_FOUND.into_response();
            }
            // Fallback to index.html for SPA routing
            match Asset::get("index.html") {
                Some(content) => {
                    let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                    ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
                }
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }
}

use std::time::Duration;

#[derive(Deserialize)]
struct AddTorrentRequest {
    uri: String,
}

async fn add_torrent_handler(
    State(state): State<AppState>,
    Json(payload): Json<AddTorrentRequest>,
) -> StatusCode {
    tracing::info!(
        "Received add_torrent request. URI length: {}",
        payload.uri.len()
    );
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
    match client.call("load.start", &["", &payload.uri]).await {
        Ok(response) => {
            tracing::debug!("rTorrent response to load.start: {}", response);
            if response.contains("faultCode") {
                tracing::error!("rTorrent returned fault: {}", response);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("Failed to add torrent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tokio::main]
async fn main() {
    // initialize tracing with env filter (default to info)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Parse CLI Args
    let args = Args::parse();
    tracing::info!("Starting VibeTorrent Backend...");
    tracing::info!("Socket: {}", args.socket);
    tracing::info!("Port: {}", args.port);

    // Channel for latest state (for new clients)
    let (tx, _rx) = watch::channel(vec![]);
    let tx = Arc::new(tx);

    // Channel for Events (Diffs)
    let (event_bus, _) = broadcast::channel::<AppEvent>(1024);

    let app_state = AppState {
        tx: tx.clone(),
        event_bus: event_bus.clone(),
        scgi_socket_path: args.socket.clone(),
    };

    // Spawn background task to poll rTorrent
    let tx_clone = tx.clone();
    let event_bus_tx = event_bus.clone();
    let socket_path = args.socket.clone(); // Clone for background task

    tokio::spawn(async move {
        let client = xmlrpc::RtorrentClient::new(&socket_path);
        let mut previous_torrents: Vec<Torrent> = Vec::new();

        loop {
            match sse::fetch_torrents(&client).await {
                Ok(new_torrents) => {
                    // 1. Update latest state (always)
                    let _ = tx_clone.send(new_torrents.clone());

                    // 2. Calculate Diff and Broadcasting
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    let mut structural_change = false;
                    if previous_torrents.len() != new_torrents.len() {
                        structural_change = true;
                    } else {
                        // Check for order/hash change
                        for (i, t) in new_torrents.iter().enumerate() {
                            if previous_torrents[i].hash != t.hash {
                                structural_change = true;
                                break;
                            }
                        }
                    }

                    if structural_change {
                        // Structural change -> Send FullList
                        let _ = event_bus_tx.send(AppEvent::FullList(new_torrents.clone(), now));
                    } else {
                        // Same structure -> Calculate partial updates
                        let updates = diff::diff_torrents(&previous_torrents, &new_torrents);
                        if !updates.is_empty() {
                            for update in updates {
                                let _ = event_bus_tx.send(update);
                            }
                        }
                    }

                    previous_torrents = new_torrents;
                }
                Err(e) => {
                    tracing::error!("Error fetching torrents in background: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let app = Router::new()
        .route("/api/events", get(sse::sse_handler))
        .route("/api/torrents/add", post(add_torrent_handler))
        .route("/api/torrents/action", post(handle_torrent_action))
        .fallback(static_handler) // Serve static files for everything else
        .layer(TraceLayer::new_for_http())
        .layer(
            CompressionLayer::new()
                .br(false)
                .gzip(true)
                .quality(CompressionLevel::Fastest),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(30))),
        )
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Backend listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn handle_torrent_action(
    State(state): State<AppState>,
    Json(payload): Json<TorrentActionRequest>,
) -> impl IntoResponse {
    tracing::info!(
        "Received action: {} for hash: {}",
        payload.action,
        payload.hash
    );

    // Special handling for delete_with_data
    if payload.action == "delete_with_data" {
        let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);

        // 1. Get Base Path
        let path_xml = match client.call("d.base_path", &[&payload.hash]).await {
            Ok(xml) => xml,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to call rTorrent: {}", e),
                )
                    .into_response()
            }
        };

        let path = match xmlrpc::parse_string_response(&path_xml) {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse path: {}", e),
                )
                    .into_response()
            }
        };

        let path_buf = std::path::Path::new(&path);

        // 1.5 Get Default Download Directory (Sandbox Root)
        let root_xml = match client.call("directory.default", &[]).await {
            Ok(xml) => xml,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get valid download root: {}", e),
                )
                    .into_response()
            }
        };

        let root_path_str = match xmlrpc::parse_string_response(&root_xml) {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse root path: {}", e),
                )
                    .into_response()
            }
        };

        // Resolve Paths (Canonicalize) to prevent .. traversal and symlink attacks
        let root_path = match std::fs::canonicalize(std::path::Path::new(&root_path_str)) {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Invalid download root configuration (on server): {}", e),
                )
                    .into_response()
            }
        };

        // Check if target path exists before trying to resolve it
        let target_path_raw = std::path::Path::new(&path);
        if !target_path_raw.exists() {
            tracing::warn!(
                "Data path not found: {:?}. Removing torrent only.",
                target_path_raw
            );
            // If file doesn't exist, we just remove the torrent entry
            if let Err(e) = client.call("d.erase", &[&payload.hash]).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to erase torrent: {}", e),
                )
                    .into_response();
            }
            return (StatusCode::OK, "Torrent removed (Data not found)").into_response();
        }

        let target_path = match std::fs::canonicalize(target_path_raw) {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Invalid data path: {}", e),
                )
                    .into_response()
            }
        };

        tracing::info!(
            "Delete request: Target='{:?}', Root='{:?}'",
            target_path,
            root_path
        );

        // SECURITY CHECK: Ensure path is inside root_path
        if !target_path.starts_with(&root_path) {
            tracing::error!(
                "Security Risk: Attempted to delete path outside download directory: {:?}",
                target_path
            );
            return (
                StatusCode::FORBIDDEN,
                "Security Error: Cannot delete files outside default download directory",
            )
                .into_response();
        }

        // SECURITY CHECK: Ensure we are not deleting the root itself
        if target_path == root_path {
            return (
                StatusCode::BAD_REQUEST,
                "Security Error: Cannot delete the download root directory itself",
            )
                .into_response();
        }

        // 2. Erase Torrent first
        if let Err(e) = client.call("d.erase", &[&payload.hash]).await {
            tracing::warn!("Failed to erase torrent entry: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to erase torrent: {}", e),
            )
                .into_response();
        }

        // 3. Delete Files via Native FS
        let delete_result = if target_path.is_dir() {
            std::fs::remove_dir_all(&target_path)
        } else {
            std::fs::remove_file(&target_path)
        };

        match delete_result {
            Ok(_) => return (StatusCode::OK, "Torrent and data deleted").into_response(),
            Err(e) => {
                tracing::error!("Failed to delete data at {:?}: {}", target_path, e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to delete data: {}", e),
                )
                    .into_response();
            }
        }
    }

    let method = match payload.action.as_str() {
        "start" => "d.start",
        "stop" => "d.stop",
        "delete" => "d.erase",
        _ => return (StatusCode::BAD_REQUEST, "Invalid action").into_response(),
    };

    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
    match client.call(method, &[&payload.hash]).await {
        Ok(_) => (StatusCode::OK, "Action executed").into_response(),
        Err(e) => {
            tracing::error!("RPC error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to execute action",
            )
                .into_response()
        }
    }
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, &'static str) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "Request timed out")
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unhandled internal error",
        )
    }
}
