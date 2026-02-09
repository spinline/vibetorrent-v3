use leptos::prelude::*;
use crate::GlobalLimitRequest;

#[server(GetGlobalLimits, "/api/server_fns")]
pub async fn get_global_limits() -> Result<GlobalLimitRequest, ServerFnError> {
    use crate::xmlrpc::{self, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);

    let down = match client.call("throttle.global_down.max_rate", &[]).await {
        Ok(xml) => xmlrpc::parse_i64_response(&xml).unwrap_or(0),
        Err(_) => -1,
    };

    let up = match client.call("throttle.global_up.max_rate", &[]).await {
        Ok(xml) => xmlrpc::parse_i64_response(&xml).unwrap_or(0),
        Err(_) => -1,
    };

    Ok(GlobalLimitRequest {
        max_download_rate: Some(down),
        max_upload_rate: Some(up),
    })
}

#[server(SetGlobalLimits, "/api/server_fns")]
pub async fn set_global_limits(
    max_download_rate: Option<i64>,
    max_upload_rate: Option<i64>,
) -> Result<(), ServerFnError> {
    use crate::xmlrpc::{RpcParam, RtorrentClient};
    let ctx = expect_context::<crate::ServerContext>();
    let client = RtorrentClient::new(&ctx.scgi_socket_path);

    if let Some(down) = max_download_rate {
        let down_kb = down / 1024;
        client
            .call(
                "throttle.global_down.max_rate.set_kb",
                &[RpcParam::from(""), RpcParam::Int(down_kb)],
            )
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to set down limit: {}", e)))?;
    }

    if let Some(up) = max_upload_rate {
        let up_kb = up / 1024;
        client
            .call(
                "throttle.global_up.max_rate.set_kb",
                &[RpcParam::from(""), RpcParam::Int(up_kb)],
            )
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to set up limit: {}", e)))?;
    }

    Ok(())
}
