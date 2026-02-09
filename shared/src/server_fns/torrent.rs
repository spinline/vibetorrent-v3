use leptos::prelude::*;
use crate::{TorrentFile, TorrentPeer, TorrentTracker};

#[server(AddTorrent, "/api/server_fns")]
pub async fn add_torrent(uri: String) -> Result<(), ServerFnError> {
    use crate::xmlrpc::{RpcParam, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);
    let params = vec![RpcParam::from(""), RpcParam::from(uri.as_str())];

    match client.call("load.start", &params).await {
        Ok(response) => {
            if response.contains("faultCode") {
                return Err(ServerFnError::new("rTorrent returned fault"));
            }
            Ok(())
        }
        Err(e) => Err(ServerFnError::new(format!("Failed to add torrent: {}", e))),
    }
}

#[server(TorrentAction, "/api/server_fns")]
pub async fn torrent_action(hash: String, action: String) -> Result<String, ServerFnError> {
    use crate::xmlrpc::{RpcParam, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);

    if action == "delete_with_data" {
        return delete_torrent_with_data_inner(&client, &hash).await;
    }

    let method = match action.as_str() {
        "start" => "d.start",
        "stop" => "d.stop",
        "delete" => "d.erase",
        _ => return Err(ServerFnError::new("Invalid action")),
    };

    let params = vec![RpcParam::from(hash.as_str())];
    match client.call(method, &params).await {
        Ok(_) => Ok("Action executed".to_string()),
        Err(e) => Err(ServerFnError::new(format!("RPC error: {}", e))),
    }
}

#[cfg(feature = "ssr")]
async fn delete_torrent_with_data_inner(
    client: &crate::xmlrpc::RtorrentClient,
    hash: &str,
) -> Result<String, ServerFnError> {
    use crate::xmlrpc::{parse_string_response, RpcParam};

    let params_hash = vec![RpcParam::from(hash)];

    let path_xml = client
        .call("d.base_path", &params_hash)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to call rTorrent: {}", e)))?;

    let path = parse_string_response(&path_xml)
        .map_err(|e| ServerFnError::new(format!("Failed to parse path: {}", e)))?;

    let root_xml = client
        .call("directory.default", &[])
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get download root: {}", e)))?;

    let root_path_str = parse_string_response(&root_xml)
        .map_err(|e| ServerFnError::new(format!("Failed to parse root path: {}", e)))?;

    let root_path = tokio::fs::canonicalize(std::path::Path::new(&root_path_str))
        .await
        .map_err(|e| ServerFnError::new(format!("Invalid download root: {}", e)))?;

    let target_path_raw = std::path::Path::new(&path);
    if !tokio::fs::try_exists(target_path_raw).await.unwrap_or(false) {
        client
            .call("d.erase", &params_hash)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to erase torrent: {}", e)))?;
        return Ok("Torrent removed (Data not found)".to_string());
    }

    let target_path = tokio::fs::canonicalize(target_path_raw)
        .await
        .map_err(|e| ServerFnError::new(format!("Invalid data path: {}", e)))?;

    if !target_path.starts_with(&root_path) {
        return Err(ServerFnError::new(
            "Security Error: Cannot delete files outside download directory",
        ));
    }

    if target_path == root_path {
        return Err(ServerFnError::new(
            "Security Error: Cannot delete the download root directory",
        ));
    }

    client
        .call("d.erase", &params_hash)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to erase torrent: {}", e)))?;

    let delete_result = if target_path.is_dir() {
        tokio::fs::remove_dir_all(&target_path).await
    } else {
        tokio::fs::remove_file(&target_path).await
    };

    match delete_result {
        Ok(_) => Ok("Torrent and data deleted".to_string()),
        Err(e) => Err(ServerFnError::new(format!("Failed to delete data: {}", e))),
    }
}

#[server(GetFiles, "/api/server_fns")]
pub async fn get_files(hash: String) -> Result<Vec<TorrentFile>, ServerFnError> {
    use crate::xmlrpc::{parse_multicall_response, RpcParam, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);
    let params = vec![
        RpcParam::from(hash.as_str()),
        RpcParam::from(""),
        RpcParam::from("f.path="),
        RpcParam::from("f.size_bytes="),
        RpcParam::from("f.completed_chunks="),
        RpcParam::from("f.priority="),
    ];

    let xml = client
        .call("f.multicall", &params)
        .await
        .map_err(|e| ServerFnError::new(format!("RPC error: {}", e)))?;

    let rows = parse_multicall_response(&xml)
        .map_err(|e| ServerFnError::new(format!("Parse error: {}", e)))?;

    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(idx, row)| TorrentFile {
            index: idx as u32,
            path: row.get(0).cloned().unwrap_or_default(),
            size: row.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
            completed_chunks: row.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
            priority: row.get(3).and_then(|s| s.parse().ok()).unwrap_or(0),
        })
        .collect())
}

#[server(GetPeers, "/api/server_fns")]
pub async fn get_peers(hash: String) -> Result<Vec<TorrentPeer>, ServerFnError> {
    use crate::xmlrpc::{parse_multicall_response, RpcParam, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);
    let params = vec![
        RpcParam::from(hash.as_str()),
        RpcParam::from(""),
        RpcParam::from("p.address="),
        RpcParam::from("p.client_version="),
        RpcParam::from("p.down_rate="),
        RpcParam::from("p.up_rate="),
        RpcParam::from("p.completed_percent="),
    ];

    let xml = client
        .call("p.multicall", &params)
        .await
        .map_err(|e| ServerFnError::new(format!("RPC error: {}", e)))?;

    let rows = parse_multicall_response(&xml)
        .map_err(|e| ServerFnError::new(format!("Parse error: {}", e)))?;

    Ok(rows
        .into_iter()
        .map(|row| TorrentPeer {
            ip: row.get(0).cloned().unwrap_or_default(),
            client: row.get(1).cloned().unwrap_or_default(),
            down_rate: row.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
            up_rate: row.get(3).and_then(|s| s.parse().ok()).unwrap_or(0),
            progress: row.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        })
        .collect())
}

#[server(GetTrackers, "/api/server_fns")]
pub async fn get_trackers(hash: String) -> Result<Vec<TorrentTracker>, ServerFnError> {
    use crate::xmlrpc::{parse_multicall_response, RpcParam, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);
    let params = vec![
        RpcParam::from(hash.as_str()),
        RpcParam::from(""),
        RpcParam::from("t.url="),
        RpcParam::from("t.activity_date_last="),
        RpcParam::from("t.message="),
    ];

    let xml = client
        .call("t.multicall", &params)
        .await
        .map_err(|e| ServerFnError::new(format!("RPC error: {}", e)))?;

    let rows = parse_multicall_response(&xml)
        .map_err(|e| ServerFnError::new(format!("Parse error: {}", e)))?;

    Ok(rows
        .into_iter()
        .map(|row| TorrentTracker {
            url: row.get(0).cloned().unwrap_or_default(),
            status: "Unknown".to_string(),
            message: row.get(2).cloned().unwrap_or_default(),
        })
        .collect())
}

#[server(SetFilePriority, "/api/server_fns")]
pub async fn set_file_priority(
    hash: String,
    file_index: u32,
    priority: u8,
) -> Result<(), ServerFnError> {
    use crate::xmlrpc::{RpcParam, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);

    let target = format!("{}:f{}", hash, file_index);
    let params = vec![
        RpcParam::from(target.as_str()),
        RpcParam::from(priority as i64),
    ];

    client
        .call("f.set_priority", &params)
        .await
        .map_err(|e| ServerFnError::new(format!("RPC error: {}", e)))?;

    let _ = client
        .call("d.update_priorities", &[RpcParam::from(hash.as_str())])
        .await;

    Ok(())
}

#[server(SetLabel, "/api/server_fns")]
pub async fn set_label(hash: String, label: String) -> Result<(), ServerFnError> {
    use crate::xmlrpc::{RpcParam, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);
    let params = vec![RpcParam::from(hash.as_str()), RpcParam::from(label)];

    client
        .call("d.custom1.set", &params)
        .await
        .map_err(|e| ServerFnError::new(format!("RPC error: {}", e)))?;

    Ok(())
}

#[server(GetVersion, "/api/server_fns")]
pub async fn get_version() -> Result<String, ServerFnError> {
    use crate::xmlrpc::{parse_string_response, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);

    match client.call("system.client_version", &[]).await {
        Ok(xml) => {
            let version = parse_string_response(&xml).unwrap_or(xml);
            Ok(version)
        }
        Err(e) => Err(ServerFnError::new(format!("Failed to get version: {}", e))),
    }
}
