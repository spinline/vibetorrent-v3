mod diff;
mod handlers;
#[cfg(feature = "push-notifications")]
mod push;
mod rate_limit;
mod sse;

use shared::xmlrpc;

use axum::error_handling::HandleErrorLayer;
use axum::{
    routing::{get, post},
    Router,
    middleware::{self, Next},
    response::Response,
    http::{StatusCode, Request},
    body::Body,
};
use axum_extra::extract::cookie::CookieJar;
use clap::Parser;
use dotenvy::dotenv;
use shared::{AppEvent, Torrent};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, watch};
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
use tower_http::{
    compression::{CompressionLayer, CompressionLevel},
    cors::CorsLayer,
    trace::TraceLayer,
};
#[cfg(feature = "swagger")]
use utoipa::OpenApi;
#[cfg(feature = "swagger")]
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub tx: Arc<watch::Sender<Vec<Torrent>>>,
    pub event_bus: broadcast::Sender<AppEvent>,
    pub scgi_socket_path: String,
    pub db: shared::db::Db,
    #[cfg(feature = "push-notifications")]
    pub push_store: push::PushSubscriptionStore,
    pub notify_poll: Arc<tokio::sync::Notify>,
}

async fn auth_middleware(
    state: axum::extract::State<AppState>,
    jar: CookieJar,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for public paths
    let path = request.uri().path();
    if path.starts_with("/api/server_fns/Login") // Login server fn
       || path.starts_with("/api/server_fns/GetSetupStatus")
       || path.starts_with("/api/server_fns/Setup")
       || path.starts_with("/swagger-ui")
       || path.starts_with("/api-docs")
       || !path.starts_with("/api/") // Allow static files (frontend)
    {
        return Ok(next.run(request).await);
    }

    // Check token
    if let Some(token) = jar.get("auth_token") {
        use jsonwebtoken::{decode, Validation, DecodingKey};
        use shared::server_fns::auth::Claims;

        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let validation = Validation::default();
        
        match decode::<Claims>(
            token.value(),
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        ) {
            Ok(_) => return Ok(next.run(request).await),
            Err(_) => {} // Invalid token
        }
    }

    Err(StatusCode::UNAUTHORIZED)
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

    /// Database URL
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:vibetorrent.db")]
    db_url: String,

    /// Reset password for the specified user
    #[arg(long)]
    reset_password: Option<String>,
}

#[cfg(feature = "swagger")]
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::login_handler,
        handlers::auth::logout_handler,
        handlers::auth::check_auth_handler,
        handlers::setup::setup_handler,
        handlers::setup::get_setup_status_handler
    ),
    components(
        schemas(
            shared::AddTorrentRequest,
            shared::TorrentActionRequest,
            shared::Torrent,
            shared::TorrentStatus,
            shared::TorrentFile,
            shared::TorrentPeer,
            shared::TorrentTracker,
            shared::SetFilePriorityRequest,
            shared::SetLabelRequest,
            shared::GlobalLimitRequest,
            handlers::auth::LoginRequest,
            handlers::setup::SetupRequest,
            handlers::setup::SetupStatusResponse,
            handlers::auth::UserResponse
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

    // Initialize Database
    tracing::info!("Connecting to database: {}", args.db_url);
    // Redundant manual creation removed, shared::db handles it

    let db: shared::db::Db = match shared::db::Db::new(&args.db_url).await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };
    tracing::info!("Database connected successfully.");

    // Handle Password Reset
    if let Some(username) = args.reset_password {
        tracing::info!("Resetting password for user: {}", username);

        // Check if user exists
        let user_result = db.get_user_by_username(&username).await;

        match user_result {
            Ok(Some((user_id, _))) => {
                // Generate random password
                use rand::{distributions::Alphanumeric, Rng};
                let new_password: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(12)
                    .map(char::from)
                    .collect();

                // Hash password (low cost for performance)
                let password_hash = match bcrypt::hash(&new_password, 6) {
                    Ok(h) => h,
                    Err(e) => {
                        tracing::error!("Failed to hash password: {}", e);
                        std::process::exit(1);
                    }
                };

                // Update in DB
                if let Err(e) = db.update_password(user_id, &password_hash).await {
                     tracing::error!("Failed to update password in DB: {}", e);
                     std::process::exit(1);
                }

                println!("--------------------------------------------------");
                println!("Password reset successfully for user: {}", username);
                println!("New Password: {}", new_password);
                println!("--------------------------------------------------");

                // Invalidate existing sessions for security
                if let Err(e) = db.delete_all_sessions_for_user(user_id).await {
                    tracing::warn!("Failed to invalidate existing sessions: {}", e);
                }

                std::process::exit(0);
            },
            Ok(None) => {
                tracing::error!("User '{}' not found.", username);
                std::process::exit(1);
            },
            Err(e) => {
                tracing::error!("Database error: {}", e);
                std::process::exit(1);
            }
        }
    }

    tracing::info!("Starting VibeTorrent Backend...");
    tracing::info!("Socket: {}", args.socket);
    tracing::info!("Port: {}", args.port);

    // ... rest of the main function ...
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

    #[cfg(feature = "push-notifications")]
    let push_store = match push::PushSubscriptionStore::with_db(&db).await {
        Ok(store) => store,
        Err(e) => {
            tracing::error!("Failed to initialize push store: {}", e);
            push::PushSubscriptionStore::new()
        }
    };

    #[cfg(not(feature = "push-notifications"))]
    let _push_store = ();

    let notify_poll = Arc::new(tokio::sync::Notify::new());

    let app_state = AppState {
        tx: tx.clone(),
        event_bus: event_bus.clone(),
        scgi_socket_path: args.socket.clone(),
        db: db.clone(),
        #[cfg(feature = "push-notifications")]
        push_store,
        notify_poll: notify_poll.clone(),
    };

    // Spawn background task to poll rTorrent
    let tx_clone = tx.clone();
    let event_bus_tx = event_bus.clone();
    let socket_path = args.socket.clone(); // Clone for background task
    #[cfg(feature = "push-notifications")]
    let push_store_clone = app_state.push_store.clone();
    let notify_poll_clone = notify_poll.clone();

    tokio::spawn(async move {
        let client = xmlrpc::RtorrentClient::new(&socket_path);
        let mut previous_torrents: Vec<Torrent> = Vec::new();
        let mut consecutive_errors = 0;
        let mut backoff_duration = Duration::from_secs(1);

        loop {
            // Determine polling interval based on active clients
            let active_clients = event_bus_tx.receiver_count();
            let loop_interval = if active_clients > 0 {
                Duration::from_secs(1)
            } else {
                Duration::from_secs(30)
            };

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
                            let _ = event_bus_tx.send(AppEvent::FullList(new_torrents.clone(), now));
                        }
                        diff::DiffResult::Partial(updates) => {
                            for update in updates {
                                // Check if this is a torrent completion notification
                                #[cfg(feature = "push-notifications")]
                                if let AppEvent::Notification(ref notif) = update {
                                    if notif.message.contains("tamamlandı") {
                                        // Send push notification in background
                                        let push_store = push_store_clone.clone();
                                        let title = "Torrent Tamamlandı".to_string();
                                        let body = notif.message.clone();
                                        tokio::spawn(async move {
                                            if let Err(e) = push::send_push_notification(
                                                &push_store,
                                                &title,
                                                &body,
                                            )
                                            .await
                                            {
                                                tracing::error!("Failed to send push notification: {}", e);
                                            }
                                        });
                                    }
                                }
                                let _ = event_bus_tx.send(update);
                            }
                        }
                        diff::DiffResult::NoChange => {}
                    }

                    previous_torrents = new_torrents;

                    // Success case: wait for the determined interval OR a wakeup notification
                    tokio::select! {
                        _ = tokio::time::sleep(loop_interval) => {},
                        _ = notify_poll_clone.notified() => {
                            tracing::debug!("Background loop awakened by new client connection");
                        }
                    }
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
                    
                    tokio::time::sleep(backoff_duration).await;
                }
            }

            // Handle Stats
            if let Ok(stats) = stats_result {
                let _ = event_bus_tx.send(AppEvent::Stats(stats));
            }
        }
    });

    let app = Router::new();

    #[cfg(feature = "swagger")]
    let app = app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    // Setup & Auth Routes (cookie-based, stay as REST)
    let scgi_path_for_ctx = args.socket.clone();
    let db_for_ctx = db.clone();
    let app = app
        .route("/api/events", get(sse::sse_handler))
        .route("/api/server_fns/{*fn_name}", post({
            let scgi_path = scgi_path_for_ctx.clone();
            let db = db_for_ctx.clone();
            move |req: Request<Body>| {
                let scgi_path = scgi_path.clone();
                let db = db.clone();
                leptos_axum::handle_server_fns_with_context(
                    move || {
                        leptos::context::provide_context(shared::ServerContext {
                            scgi_socket_path: scgi_path.clone(),
                        });
                        leptos::context::provide_context(shared::DbContext {
                            db: db.clone(),
                        });
                    },
                    req,
                )
            }
        }))
        .fallback(handlers::static_handler);

    let app = app
        .layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware))
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
    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
