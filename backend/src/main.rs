mod diff;
mod handlers;
mod scgi;
mod sse;
mod xmlrpc;

use axum::error_handling::HandleErrorLayer;
use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use dotenvy::dotenv;
use shared::{AppEvent, Torrent};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
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
    #[arg(
        short,
        long,
        env = "RTORRENT_SOCKET",
        default_value = "/tmp/rtorrent.sock"
    )]
    socket: String,

    /// Port to listen on
    #[arg(short, long, env = "PORT", default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() {
    // Load .env file
    let _ = dotenv();

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

    // Startup Health Check
    let socket_path = std::path::Path::new(&args.socket);
    if !socket_path.exists() {
        tracing::error!("CRITICAL: rTorrent socket not found at {:?}.", socket_path);
        tracing::warn!(
            "HINT: Make sure rTorrent is running and the SCGI socket is enabled in .rtorrent.rc"
        );
        tracing::warn!(
            "HINT: You can configure the socket path via --socket ARG or RTORRENT_SOCKET ENV."
        );
    } else {
        tracing::info!("Socket file exists. Testing connection...");
        let client = xmlrpc::RtorrentClient::new(&args.socket);
        // We use a lightweight call to verify connectivity
        match client.call("system.client_version", &[]).await {
            Ok(v) => tracing::info!("Connected to rTorrent successfully. Version: {}", v),
            Err(e) => tracing::error!("Socket exists but failed to connect to rTorrent: {}", e),
        }
    }

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

                    match diff::diff_torrents(&previous_torrents, &new_torrents) {
                        diff::DiffResult::FullUpdate => {
                            let _ =
                                event_bus_tx.send(AppEvent::FullList(new_torrents.clone(), now));
                        }
                        diff::DiffResult::Partial(updates) => {
                            for update in updates {
                                let _ = event_bus_tx.send(update);
                            }
                        }
                        diff::DiffResult::NoChange => {}
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
        .route("/api/torrents/add", post(handlers::add_torrent_handler))
        .route(
            "/api/torrents/action",
            post(handlers::handle_torrent_action),
        )
        .fallback(handlers::static_handler) // Serve static files for everything else
        .layer(TraceLayer::new_for_http())
        .layer(
            CompressionLayer::new()
                .br(false)
                .gzip(true)
                .quality(CompressionLevel::Fastest),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handlers::handle_timeout_error))
                .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(30))),
        )
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Backend listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
