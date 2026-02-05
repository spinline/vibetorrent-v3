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
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::add_torrent_handler,
        handlers::handle_torrent_action,
        handlers::get_version_handler,
        handlers::get_files_handler,
        handlers::get_peers_handler,
        handlers::get_trackers_handler,
        handlers::set_file_priority_handler,
        handlers::set_label_handler,
        handlers::get_global_limit_handler,
        handlers::set_global_limit_handler
    ),
    components(
        schemas(
            handlers::AddTorrentRequest,
            shared::TorrentActionRequest,
            shared::Torrent,
            shared::TorrentStatus,
            shared::TorrentFile,
            shared::TorrentPeer,
            shared::TorrentTracker,
            shared::SetFilePriorityRequest,
            shared::SetLabelRequest,
            shared::GlobalLimitRequest
        )
    ),
    tags(
        (name = "vibetorrent", description = "VibeTorrent API")
    )
)]
struct ApiDoc;

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
        let params: Vec<xmlrpc::RpcParam> = vec![];
        match client.call("system.client_version", &params).await {
            Ok(xml) => {
                let version = xmlrpc::parse_string_response(&xml).unwrap_or(xml);
                tracing::info!("Connected to rTorrent successfully. Version: {}", version);
            }
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
        let mut consecutive_errors = 0;
        let mut backoff_duration = Duration::from_secs(1);

        loop {
            // 1. Fetch Torrents
            let torrents_result = sse::fetch_torrents(&client).await;

            // 2. Fetch Global Stats
            let stats_result = sse::fetch_global_stats(&client).await;

            // Handle Torrents
            match torrents_result {
                Ok(new_torrents) => {
                    // Check if we recovered from an error state
                    if consecutive_errors > 0 {
                        tracing::info!(
                            "Reconnected to rTorrent after {} failures.",
                            consecutive_errors
                        );
                        let _ =
                            event_bus_tx.send(AppEvent::Notification(shared::SystemNotification {
                                level: shared::NotificationLevel::Success,
                                message: "Reconnected to rTorrent".to_string(),
                            }));
                        consecutive_errors = 0;
                        backoff_duration = Duration::from_secs(1);
                    }

                    // Update latest state
                    let _ = tx_clone.send(new_torrents.clone());

                    // Calculate Diff and Broadcasting
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    match diff::diff_torrents(&previous_torrents, &new_torrents) {
                        diff::DiffResult::FullUpdate => {
                            let _ = event_bus_tx.send(AppEvent::FullList {
                                torrents: new_torrents.clone(),
                                timestamp: now,
                            });
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
                    consecutive_errors += 1;

                    // If this is the first error after success (or startup), notify clients
                    if consecutive_errors == 1 {
                        let _ =
                            event_bus_tx.send(AppEvent::Notification(shared::SystemNotification {
                                level: shared::NotificationLevel::Error,
                                message: format!("Lost connection to rTorrent: {}", e),
                            }));
                    }

                    // Exponential backoff with a cap of 30 seconds
                    backoff_duration = std::cmp::min(backoff_duration * 2, Duration::from_secs(30));
                    tracing::warn!(
                        "Backoff: Sleeping for {:?} due to rTorrent error.",
                        backoff_duration
                    );
                }
            }

            // Handle Stats
            match stats_result {
                Ok(stats) => {
                    let _ = event_bus_tx.send(AppEvent::Stats(stats));
                }
                Err(e) => {
                    tracing::warn!("Error fetching global stats: {}", e);
                }
            }

            tokio::time::sleep(backoff_duration).await;
        }
    });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/events", get(sse::sse_handler))
        .route("/api/torrents/add", post(handlers::add_torrent_handler))
        .route(
            "/api/torrents/action",
            post(handlers::handle_torrent_action),
        )
        .route("/api/system/version", get(handlers::get_version_handler))
        .route(
            "/api/torrents/{hash}/files",
            get(handlers::get_files_handler),
        )
        .route(
            "/api/torrents/{hash}/peers",
            get(handlers::get_peers_handler),
        )
        .route(
            "/api/torrents/{hash}/trackers",
            get(handlers::get_trackers_handler),
        )
        .route(
            "/api/torrents/files/priority",
            post(handlers::set_file_priority_handler),
        )
        .route("/api/torrents/label", post(handlers::set_label_handler))
        .route(
            "/api/settings/global-limits",
            get(handlers::get_global_limit_handler).post(handlers::set_global_limit_handler),
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
    tracing::info!("Backend attempting to listen on {}", addr);
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("FATAL: Failed to bind to address {}: {}", addr, e);
            if e.kind() == std::io::ErrorKind::AddrInUse {
                tracing::error!("HINT: Port {} is already in use. Stop the existing process or use --port to specify a different port.", args.port);
            }
            std::process::exit(1);
        }
    };
    tracing::info!("Backend listening on {}", addr);
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
