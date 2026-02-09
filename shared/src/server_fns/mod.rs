use leptos::*;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::xmlrpc::{self, RtorrentClient};

#[server(GetVersion, "/api/server_fns")]
pub async fn get_version() -> Result<String, ServerFnError> {
    let socket_path = std::env::var("RTORRENT_SOCKET").unwrap_or_else(|_| "/tmp/rtorrent.sock".to_string());
    
    #[cfg(feature = "ssr")]
    {
        let client = RtorrentClient::new(&socket_path);
        match client.call("system.client_version", &[]).await {
            Ok(xml) => {
                let version = xmlrpc::parse_string_response(&xml).unwrap_or(xml);
                Ok(version)
            },
            Err(e) => Err(ServerFnError::ServerError(e.to_string())),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}