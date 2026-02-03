use crate::{
    xmlrpc::{self, RpcParam},
    AppState,
};
use axum::{
    extract::{Json, Path, State},
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    BoxError,
};
use rust_embed::RustEmbed;
use serde::Deserialize;
use shared::{
    GlobalLimitRequest, SetFilePriorityRequest, SetLabelRequest, TorrentActionRequest, TorrentFile,
    TorrentPeer, TorrentTracker,
};
use utoipa::ToSchema;

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
pub struct Asset;

#[derive(Deserialize, ToSchema)]
pub struct AddTorrentRequest {
    /// Magnet link or Torrent file URL
    #[schema(example = "magnet:?xt=urn:btih:...")]
    uri: String,
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
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

// --- TORRENT ACTIONS ---

/// Add a new torrent via magnet link or URL
#[utoipa::path(
    post,
    path = "/api/torrents/add",
    request_body = AddTorrentRequest,
    responses(
        (status = 200, description = "Torrent added successfully"),
        (status = 500, description = "Internal server error or rTorrent fault")
    )
)]
pub async fn add_torrent_handler(
    State(state): State<AppState>,
    Json(payload): Json<AddTorrentRequest>,
) -> StatusCode {
    tracing::info!(
        "Received add_torrent request. URI length: {}",
        payload.uri.len()
    );
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
    let params = vec![RpcParam::from(""), RpcParam::from(payload.uri.as_str())];

    match client.call("load.start", &params).await {
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

/// Perform an action on a torrent (start, stop, delete)
#[utoipa::path(
    post,
    path = "/api/torrents/action",
    request_body = TorrentActionRequest,
    responses(
        (status = 200, description = "Action executed successfully"),
        (status = 400, description = "Invalid action or request"),
        (status = 403, description = "Forbidden: Security risk detected"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn handle_torrent_action(
    State(state): State<AppState>,
    Json(payload): Json<TorrentActionRequest>,
) -> impl IntoResponse {
    tracing::info!(
        "Received action: {} for hash: {}",
        payload.action,
        payload.hash
    );

    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);

    // Special handling for delete_with_data
    if payload.action == "delete_with_data" {
        return match delete_torrent_with_data(&client, &payload.hash).await {
            Ok(msg) => (StatusCode::OK, msg).into_response(),
            Err((status, msg)) => (status, msg).into_response(),
        };
    }

    let method = match payload.action.as_str() {
        "start" => "d.start",
        "stop" => "d.stop",
        "delete" => "d.erase",
        _ => return (StatusCode::BAD_REQUEST, "Invalid action").into_response(),
    };

    let params = vec![RpcParam::from(payload.hash.as_str())];

    match client.call(method, &params).await {
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

/// Helper function to handle secure deletion of torrent data
async fn delete_torrent_with_data(
    client: &xmlrpc::RtorrentClient,
    hash: &str,
) -> Result<&'static str, (StatusCode, String)> {
    let params_hash = vec![RpcParam::from(hash)];

    // 1. Get Base Path
    let path_xml = client
        .call("d.base_path", &params_hash)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to call rTorrent: {}", e),
            )
        })?;

    let path = xmlrpc::parse_string_response(&path_xml).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse path: {}", e),
        )
    })?;

    // 1.5 Get Default Download Directory (Sandbox Root)
    let root_xml = client.call("directory.default", &[]).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get valid download root: {}", e),
        )
    })?;

    let root_path_str = xmlrpc::parse_string_response(&root_xml).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse root path: {}", e),
        )
    })?;

    // Resolve Paths (Canonicalize) to prevent .. traversal and symlink attacks
    let root_path = std::fs::canonicalize(std::path::Path::new(&root_path_str)).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Invalid download root configuration (on server): {}", e),
        )
    })?;

    // Check if target path exists before trying to resolve it
    let target_path_raw = std::path::Path::new(&path);
    if !target_path_raw.exists() {
        tracing::warn!(
            "Data path not found: {:?}. Removing torrent only.",
            target_path_raw
        );
        // If file doesn't exist, we just remove the torrent entry
        client.call("d.erase", &params_hash).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to erase torrent: {}", e),
            )
        })?;

        return Ok("Torrent removed (Data not found)");
    }

    let target_path = std::fs::canonicalize(target_path_raw).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Invalid data path: {}", e),
        )
    })?;

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
        return Err((
            StatusCode::FORBIDDEN,
            "Security Error: Cannot delete files outside default download directory".to_string(),
        ));
    }

    // SECURITY CHECK: Ensure we are not deleting the root itself
    if target_path == root_path {
        return Err((
            StatusCode::BAD_REQUEST,
            "Security Error: Cannot delete the download root directory itself".to_string(),
        ));
    }

    // 2. Erase Torrent first
    client.call("d.erase", &params_hash).await.map_err(|e| {
        tracing::warn!("Failed to erase torrent entry: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to erase torrent: {}", e),
        )
    })?;

    // 3. Delete Files via Native FS
    let delete_result = if target_path.is_dir() {
        std::fs::remove_dir_all(&target_path)
    } else {
        std::fs::remove_file(&target_path)
    };

    match delete_result {
        Ok(_) => Ok("Torrent and data deleted"),
        Err(e) => {
            tracing::error!("Failed to delete data at {:?}: {}", target_path, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to delete data: {}", e),
            ))
        }
    }
}

// --- NEW HANDLERS ---

/// Get rTorrent version
#[utoipa::path(
    get,
    path = "/api/system/version",
    responses(
        (status = 200, description = "rTorrent version", body = String),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_version_handler(State(state): State<AppState>) -> impl IntoResponse {
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
    match client.call("system.client_version", &[]).await {
        Ok(xml) => {
            let version = xmlrpc::parse_string_response(&xml).unwrap_or(xml);
            (StatusCode::OK, version).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get version: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get version").into_response()
        }
    }
}

/// Get files for a torrent
#[utoipa::path(
    get,
    path = "/api/torrents/{hash}/files",
    responses(
        (status = 200, description = "Files list", body = Vec<TorrentFile>),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("hash" = String, Path, description = "Torrent Hash")
    )
)]
pub async fn get_files_handler(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
    let params = vec![
        RpcParam::from(hash.as_str()),
        RpcParam::from(""),
        RpcParam::from("f.path="),
        RpcParam::from("f.size_bytes="),
        RpcParam::from("f.completed_chunks="),
        RpcParam::from("f.priority="),
    ];

    match client.call("f.multicall", &params).await {
        Ok(xml) => match xmlrpc::parse_multicall_response(&xml) {
            Ok(rows) => {
                let files: Vec<TorrentFile> = rows
                    .into_iter()
                    .enumerate()
                    .map(|(idx, row)| TorrentFile {
                        index: idx as u32,
                        path: row.get(0).cloned().unwrap_or_default(),
                        size: row.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                        completed_chunks: row.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
                        priority: row.get(3).and_then(|s| s.parse().ok()).unwrap_or(0),
                    })
                    .collect();
                (StatusCode::OK, Json(files)).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Parse error: {}", e),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("RPC error: {}", e),
        )
            .into_response(),
    }
}

/// Get peers for a torrent
#[utoipa::path(
    get,
    path = "/api/torrents/{hash}/peers",
    responses(
        (status = 200, description = "Peers list", body = Vec<TorrentPeer>),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("hash" = String, Path, description = "Torrent Hash")
    )
)]
pub async fn get_peers_handler(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
    let params = vec![
        RpcParam::from(hash.as_str()),
        RpcParam::from(""),
        RpcParam::from("p.address="),
        RpcParam::from("p.client_version="),
        RpcParam::from("p.down_rate="),
        RpcParam::from("p.up_rate="),
        RpcParam::from("p.completed_percent="),
    ];

    match client.call("p.multicall", &params).await {
        Ok(xml) => match xmlrpc::parse_multicall_response(&xml) {
            Ok(rows) => {
                let peers: Vec<TorrentPeer> = rows
                    .into_iter()
                    .map(|row| TorrentPeer {
                        ip: row.get(0).cloned().unwrap_or_default(),
                        client: row.get(1).cloned().unwrap_or_default(),
                        down_rate: row.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
                        up_rate: row.get(3).and_then(|s| s.parse().ok()).unwrap_or(0),
                        progress: row.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                    })
                    .collect();
                (StatusCode::OK, Json(peers)).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Parse error: {}", e),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("RPC error: {}", e),
        )
            .into_response(),
    }
}

/// Get trackers for a torrent
#[utoipa::path(
    get,
    path = "/api/torrents/{hash}/trackers",
    responses(
        (status = 200, description = "Trackers list", body = Vec<TorrentTracker>),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("hash" = String, Path, description = "Torrent Hash")
    )
)]
pub async fn get_trackers_handler(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
    let params = vec![
        RpcParam::from(hash.as_str()),
        RpcParam::from(""),
        RpcParam::from("t.url="),
        RpcParam::from("t.activity_date_last="),
        RpcParam::from("t.message="),
    ];

    match client.call("t.multicall", &params).await {
        Ok(xml) => {
            match xmlrpc::parse_multicall_response(&xml) {
                Ok(rows) => {
                    let trackers: Vec<TorrentTracker> = rows
                        .into_iter()
                        .map(|row| {
                            TorrentTracker {
                                url: row.get(0).cloned().unwrap_or_default(),
                                status: "Unknown".to_string(), // Derive from type/activity?
                                message: row.get(2).cloned().unwrap_or_default(),
                            }
                        })
                        .collect();
                    (StatusCode::OK, Json(trackers)).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Parse error: {}", e),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("RPC error: {}", e),
        )
            .into_response(),
    }
}

/// Set file priority
#[utoipa::path(
    post,
    path = "/api/torrents/files/priority",
    request_body = SetFilePriorityRequest,
    responses(
        (status = 200, description = "Priority updated"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn set_file_priority_handler(
    State(state): State<AppState>,
    Json(payload): Json<SetFilePriorityRequest>,
) -> impl IntoResponse {
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);

    // f.set_priority takes "hash", index, priority
    // Priority: 0 (off), 1 (normal), 2 (high)
    // f.set_priority is tricky. Let's send as string first as before, or int if we knew.
    // Usually priorities are small integers.
    // But since we are updating everything to RpcParam, let's use Int if possible or String.
    // The previous implementation used string. Let's stick to string for now or try Int.
    // Actually, f.set_priority likely takes an integer.

    let target = format!("{}:f{}", payload.hash, payload.file_index);
    let params = vec![
        RpcParam::from(target.as_str()),
        RpcParam::from(payload.priority as i64),
    ];

    match client.call("f.set_priority", &params).await {
        Ok(_) => {
            let _ = client
                .call(
                    "d.update_priorities",
                    &[RpcParam::from(payload.hash.as_str())],
                )
                .await;
            (StatusCode::OK, "Priority updated").into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("RPC error: {}", e),
        )
            .into_response(),
    }
}

/// Set torrent label
#[utoipa::path(
    post,
    path = "/api/torrents/label",
    request_body = SetLabelRequest,
    responses(
        (status = 200, description = "Label updated"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn set_label_handler(
    State(state): State<AppState>,
    Json(payload): Json<SetLabelRequest>,
) -> impl IntoResponse {
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
    let params = vec![
        RpcParam::from(payload.hash.as_str()),
        RpcParam::from(payload.label),
    ];

    match client.call("d.custom1.set", &params).await {
        Ok(_) => (StatusCode::OK, "Label updated").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("RPC error: {}", e),
        )
            .into_response(),
    }
}

/// Get global speed limits
#[utoipa::path(
    get,
    path = "/api/settings/global-limits",
    responses(
        (status = 200, description = "Current limits", body = GlobalLimitRequest),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_global_limit_handler(State(state): State<AppState>) -> impl IntoResponse {
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);
    // throttle.global_down.max_rate, throttle.global_up.max_rate
    let down_fut = client.call("throttle.global_down.max_rate", &[]);
    let up_fut = client.call("throttle.global_up.max_rate", &[]);

    let down = match down_fut.await {
        Ok(xml) => xmlrpc::parse_i64_response(&xml).unwrap_or(0),
        Err(_) => -1,
    };

    let up = match up_fut.await {
        Ok(xml) => xmlrpc::parse_i64_response(&xml).unwrap_or(0),
        Err(_) => -1,
    };

    let resp = GlobalLimitRequest {
        max_download_rate: Some(down),
        max_upload_rate: Some(up),
    };

    (StatusCode::OK, Json(resp)).into_response()
}

/// Set global speed limits
#[utoipa::path(
    post,
    path = "/api/settings/global-limits",
    request_body = GlobalLimitRequest,
    responses(
        (status = 200, description = "Limits updated"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn set_global_limit_handler(
    State(state): State<AppState>,
    Json(payload): Json<GlobalLimitRequest>,
) -> impl IntoResponse {
    let client = xmlrpc::RtorrentClient::new(&state.scgi_socket_path);

    if let Some(down) = payload.max_download_rate {
        // Here is the fix: Send as Int
        if let Err(e) = client
            .call("throttle.global_down.max_rate.set", &[RpcParam::Int(down)])
            .await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to set down limit: {}", e),
            )
                .into_response();
        }
    }

    if let Some(up) = payload.max_upload_rate {
        if let Err(e) = client
            .call("throttle.global_up.max_rate.set", &[RpcParam::Int(up)])
            .await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to set up limit: {}", e),
            )
                .into_response();
        }
    }

    (StatusCode::OK, "Limits updated").into_response()
}

pub async fn handle_timeout_error(err: BoxError) -> (StatusCode, &'static str) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "Request timed out")
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unhandled internal error",
        )
    }
}
