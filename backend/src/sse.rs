use shared::xmlrpc::{
    parse_i64_response, parse_multicall_response, RpcParam, RtorrentClient, XmlRpcError,
};
use crate::AppState;
use axum::extract::State;
use axum::response::sse::{Event, Sse};
use futures::stream::{self, Stream};
use shared::{AppEvent, GlobalStats, Torrent, TorrentStatus};
use std::convert::Infallible;
use tokio_stream::StreamExt;
use axum::response::IntoResponse;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

// Field definitions to keep query and parser in sync
mod fields {
    pub const IDX_HASH: usize = 0;
    pub const CMD_HASH: &str = "d.hash=";

    pub const IDX_NAME: usize = 1;
    pub const CMD_NAME: &str = "d.name=";

    pub const IDX_SIZE: usize = 2;
    pub const CMD_SIZE: &str = "d.size_bytes=";

    pub const IDX_COMPLETED: usize = 3;
    pub const CMD_COMPLETED: &str = "d.bytes_done=";

    pub const IDX_DOWN_RATE: usize = 4;
    pub const CMD_DOWN_RATE: &str = "d.down.rate=";

    pub const IDX_UP_RATE: usize = 5;
    pub const CMD_UP_RATE: &str = "d.up.rate=";

    pub const IDX_STATE: usize = 6;
    pub const CMD_STATE: &str = "d.state=";

    pub const IDX_COMPLETE: usize = 7;
    pub const CMD_COMPLETE: &str = "d.complete=";

    pub const IDX_MESSAGE: usize = 8;
    pub const CMD_MESSAGE: &str = "d.message=";

    pub const IDX_LEFT_BYTES: usize = 9;
    pub const CMD_LEFT_BYTES: &str = "d.left_bytes=";

    pub const IDX_CREATION_DATE: usize = 10;
    pub const CMD_CREATION_DATE: &str = "d.creation_date=";

    pub const IDX_HASHING: usize = 11;
    pub const CMD_HASHING: &str = "d.hashing=";

    pub const IDX_LABEL: usize = 12;
    pub const CMD_LABEL: &str = "d.custom1=";
}

use fields::*;

// Constants for rTorrent fields to ensure query and parser stay in sync
const RTORRENT_FIELDS: &[&str] = &[
    "",     // Ignored by multicall pattern
    "main", // View
    CMD_HASH,
    CMD_NAME,
    CMD_SIZE,
    CMD_COMPLETED,
    CMD_DOWN_RATE,
    CMD_UP_RATE,
    CMD_STATE,
    CMD_COMPLETE,
    CMD_MESSAGE,
    CMD_LEFT_BYTES,
    CMD_CREATION_DATE,
    CMD_HASHING,
    CMD_LABEL,
];

fn parse_long(s: Option<&String>) -> i64 {
    s.map(|v| v.parse().unwrap_or(0)).unwrap_or(0)
}

fn parse_string(s: Option<&String>) -> String {
    s.cloned().unwrap_or_default()
}

/// Converts a raw row of strings from rTorrent XML-RPC into a generic Torrent struct
fn from_rtorrent_row(row: Vec<String>) -> Torrent {
    let hash = parse_string(row.get(IDX_HASH));
    let name = parse_string(row.get(IDX_NAME));
    let size = parse_long(row.get(IDX_SIZE));
    let completed = parse_long(row.get(IDX_COMPLETED));
    let down_rate = parse_long(row.get(IDX_DOWN_RATE));
    let up_rate = parse_long(row.get(IDX_UP_RATE));

    let state = parse_long(row.get(IDX_STATE));
    let is_complete = parse_long(row.get(IDX_COMPLETE));
    let message = parse_string(row.get(IDX_MESSAGE));
    let left_bytes = parse_long(row.get(IDX_LEFT_BYTES));
    let added_date = parse_long(row.get(IDX_CREATION_DATE));
    let is_hashing = parse_long(row.get(IDX_HASHING));
    let label_raw = parse_string(row.get(IDX_LABEL));

    let label = if label_raw.is_empty() {
        None
    } else {
        Some(label_raw)
    };

    let percent_complete = if size > 0 {
        (completed as f64 / size as f64) * 100.0
    } else {
        0.0
    };

    // Status Logic
    let status = if !message.is_empty() {
        TorrentStatus::Error
    } else if is_hashing != 0 {
        TorrentStatus::Checking
    } else if state == 0 {
        TorrentStatus::Paused
    } else if is_complete != 0 {
        TorrentStatus::Seeding
    } else {
        TorrentStatus::Downloading
    };

    // ETA Logic (seconds)
    let eta = if down_rate > 0 && left_bytes > 0 {
        left_bytes / down_rate
    } else {
        0
    };

    Torrent {
        hash,
        name,
        size,
        completed,
        down_rate,
        up_rate,
        eta,
        percent_complete,
        status,
        error_message: message,
        added_date,
        label,
    }
}

pub async fn fetch_torrents(client: &RtorrentClient) -> Result<Vec<Torrent>, XmlRpcError> {
    let params: Vec<RpcParam> = RTORRENT_FIELDS.iter().map(|s| RpcParam::from(*s)).collect();
    let xml = client.call("d.multicall2", &params).await?;

    if xml.trim().is_empty() {
        return Err(XmlRpcError::Parse("Empty response from SCGI".to_string()));
    }

    let rows = parse_multicall_response(&xml)?;

    let torrents = rows.into_iter().map(from_rtorrent_row).collect();

    Ok(torrents)
}

pub async fn fetch_global_stats(client: &RtorrentClient) -> Result<GlobalStats, XmlRpcError> {
    let empty_params: Vec<RpcParam> = vec![];

    let down_rate_xml = client
        .call("throttle.global_down.rate", &empty_params)
        .await?;
    let down_rate = parse_i64_response(&down_rate_xml).unwrap_or(0);

    let up_rate_xml = client
        .call("throttle.global_up.rate", &empty_params)
        .await?;
    let up_rate = parse_i64_response(&up_rate_xml).unwrap_or(0);

    let down_limit_xml = client
        .call("throttle.global_down.max_rate", &empty_params)
        .await?;
    let down_limit = parse_i64_response(&down_limit_xml).ok();

    let up_limit_xml = client
        .call("throttle.global_up.max_rate", &empty_params)
        .await?;
    let up_limit = parse_i64_response(&up_limit_xml).ok();

    Ok(GlobalStats {
        down_rate,
        up_rate,
        down_limit,
        up_limit,
        free_space: None,
    })
}

pub async fn sse_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Notify background worker to wake up and poll immediately
    state.notify_poll.notify_one();

    // Get initial value synchronously (from the watch channel's current state)
    let initial_rx = state.tx.subscribe();
    let initial_torrents = initial_rx.borrow().clone();

    let initial_event = {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let event_data = AppEvent::FullList(initial_torrents, timestamp);

        match rmp_serde::to_vec(&event_data) {
            Ok(bytes) => Event::default().data(BASE64.encode(bytes)),
            Err(_) => Event::default().comment("init_error"),
        }
    };

    // Stream that yields the initial event once
    let initial_stream = stream::once(async { Ok::<Event, Infallible>(initial_event) });

    // Stream that waits for subsequent changes via Broadcast channel
    let rx = state.event_bus.subscribe();
    let update_stream = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(event) => match rmp_serde::to_vec(&event) {
                Ok(bytes) => Some((Ok::<Event, Infallible>(Event::default().data(BASE64.encode(bytes))), rx)),
                Err(e) => {
                    tracing::warn!("Failed to serialize SSE event (MessagePack): {}", e);
                    Some((
                        Ok::<Event, Infallible>(Event::default().comment("error")),
                        rx,
                    ))
                }
            },
            Err(e) => {
                // If channel closed or lagged, close stream so client reconnects and gets fresh state
                tracing::warn!("SSE Broadcast channel error (lagged/closed): {}", e);
                None
            }
        }
    });

    let sse = Sse::new(initial_stream.chain(update_stream))
        .keep_alive(axum::response::sse::KeepAlive::default());

    (
        [("content-type", "text/event-stream")],
        sse
    )
}