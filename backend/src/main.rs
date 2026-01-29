
mod models;
mod scgi;
mod sse;
mod xmlrpc;

// fixup modules
// remove mm if I didn't create it? I didn't. 
// I will structure modules correctly.

use clap::Parser;
use rust_embed::RustEmbed;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router, Json,
};
use tower_http::cors::CorsLayer;
use serde::Deserialize;
use std::net::SocketAddr;
use crate::models::AppState;

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

use tokio::sync::watch;
use std::sync::Arc;
use std::time::Duration;

/* ... add_torrent_handler ... */

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    
    // Parse CLI Args
    let args = Args::parse();
    println!("Starting VibeTorrent Backend...");
    println!("Socket: {}", args.socket);
    println!("Port: {}", args.port);
    
    // Channel for torrent list updates
    let (tx, _rx) = watch::channel(vec![]);
    let tx = Arc::new(tx);
    
    let app_state = AppState {
        tx: tx.clone(),
        scgi_socket_path: args.socket.clone(),
    };

    // Spawn background task to poll rTorrent
    let tx_clone = tx.clone();
    let socket_path = args.socket.clone(); // Clone for background task
    tokio::spawn(async move {
        let client = xmlrpc::RtorrentClient::new(&socket_path);
        loop {
            match sse::fetch_torrents(&client).await {
                Ok(torrents) => {
                    let _ = tx_clone.send(torrents);
                }
                Err(e) => {
                    eprintln!("Error fetching torrents in background: {}", e);
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
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Backend listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn handle_torrent_action(
    State(state): State<AppState>,
    Json(payload): Json<models::TorrentActionRequest>,
) -> impl IntoResponse {
    println!("Received action: {} for hash: {}", payload.action, payload.hash);
    
    // Special handling for delete_with_data
    if payload.action == "delete_with_data" {
        let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
        
        // 1. Get Base Path
        let path_xml = match client.call("d.base_path", &[&payload.hash]).await {
            Ok(xml) => xml,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to call rTorrent: {}", e)).into_response(),
        };

        let path = match xmlrpc::parse_string_response(&path_xml) {
            Ok(p) => p,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse path: {}", e)).into_response(),
        };
        
        println!("Attempting to delete torrent and data at path: {}", path);
        if path.trim().is_empty() || path == "/" {
             return (StatusCode::BAD_REQUEST, "Safety check failed: Path is empty or root").into_response();
        }

        // 2. Erase Torrent first (so rTorrent releases locks?)
        if let Err(e) = client.call("d.erase", &[&payload.hash]).await {
             eprintln!("Failed to erase torrent entry: {}", e);
             // Proceed anyway to delete files? Maybe not.
        }

        // 3. Delete Files via rTorrent (execute.throw.bg)
        // Command: rm -rf <path>
        match client.call("execute.throw.bg", &["", "rm", "-rf", &path]).await {
            Ok(_) => return (StatusCode::OK, "Torrent and data deleted").into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete data: {}", e)).into_response(),
        }
    }

    let method = match payload.action.as_str() {
        "start" => "d.start",
        "stop" => "d.stop",
        "delete" => "d.erase",
        _ => return (StatusCode::BAD_REQUEST, "Invalid action").into_response(),
    };

    match scgi::system_call(&state.scgi_socket_path, method, vec![&payload.hash]).await {
        Ok(_) => (StatusCode::OK, "Action executed").into_response(),
        Err(e) => {
            eprintln!("SCGI error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to execute action").into_response()
        }
    }
}
