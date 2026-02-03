use crate::xmlrpc::{parse_multicall_response, RtorrentClient, XmlRpcError};
use crate::AppState;
use axum::extract::State;
use axum::response::sse::{Event, Sse};
use futures::stream::{self, Stream};
use shared::{AppEvent, GlobalStats, Torrent, TorrentStatus};
use std::convert::Infallible;
use tokio_stream::StreamExt;

// Constants for rTorrent fields to ensure query and parser stay in sync
const RTORRENT_FIELDS: &[&str] = &[
    "",                 // 0: default (ignored)
    "main",             // 1: view
    "d.hash=",          // 0 -> row index starts after view
    "d.name=",          // 1
    "d.size_bytes=",    // 2
    "d.bytes_done=",    // 3
    "d.down.rate=",     // 4
    "d.up.rate=",       // 5
    "d.state=",         // 6
    "d.complete=",      // 7
    "d.message=",       // 8
    "d.left_bytes=",    // 9
    "d.creation_date=", // 10
    "d.hashing=",       // 11
    "d.custom1=",       // 12 (Label)
];

fn parse_long(s: Option<&String>) -> i64 {
    s.map(|v| v.parse().unwrap_or(0)).unwrap_or(0)
}

fn parse_string(s: Option<&String>) -> String {
    s.cloned().unwrap_or_default()
}

/// Converts a raw row of strings from rTorrent XML-RPC into a generic Torrent struct
fn from_rtorrent_row(row: Vec<String>) -> Torrent {
    // Indexes correspond to the params list below (excluding the first two view/target args)
    let hash = parse_string(row.get(0));
    let name = parse_string(row.get(1));
    let size = parse_long(row.get(2));
    let completed = parse_long(row.get(3));
    let down_rate = parse_long(row.get(4));
    let up_rate = parse_long(row.get(5));

    let state = parse_long(row.get(6));
    let is_complete = parse_long(row.get(7));
    let message = parse_string(row.get(8));
    let left_bytes = parse_long(row.get(9));
    let added_date = parse_long(row.get(10));
    let is_hashing = parse_long(row.get(11));
    let label_raw = parse_string(row.get(12));

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
    let xml = client.call("d.multicall2", RTORRENT_FIELDS).await?;

    if xml.trim().is_empty() {
        return Err(XmlRpcError::Parse("Empty response from SCGI".to_string()));
    }

    let rows = parse_multicall_response(&xml)?;

    let torrents = rows.into_iter().map(from_rtorrent_row).collect();

    Ok(torrents)
}

pub async fn fetch_global_stats(client: &RtorrentClient) -> Result<GlobalStats, XmlRpcError> {
    // Parallel calls would be better but let's keep it simple sequential for now.
    // NOTE: This adds 4 roundtrips per second. If this is too slow, we should use multicall via system.multicall (if supported)
    // or just accept the overhead. Unix socket overhead is very low.

    // We ignore errors on individual stats to not break the whole loop, using defaults.
    // But connection errors should propagate.

    let down_rate_str = client.call("throttle.global_down.rate", &[]).await?;
    let up_rate_str = client.call("throttle.global_up.rate", &[]).await?;
    let down_limit_str = client.call("throttle.global_down.max_rate", &[]).await?;
    let up_limit_str = client.call("throttle.global_up.max_rate", &[]).await?;

    // Optionally get free space. "directory.default" then "d.free_space_path"?? No "get_directory_free_space"
    // Let's skip free space for high frequency updates.

    Ok(GlobalStats {
        down_rate: down_rate_str.parse().unwrap_or(0),
        up_rate: up_rate_str.parse().unwrap_or(0),
        down_limit: down_limit_str.parse().ok(),
        up_limit: up_limit_str.parse().ok(),
        free_space: None,
    })
}

pub async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Get initial value synchronously (from the watch channel's current state)
    let initial_rx = state.tx.subscribe();
    let initial_torrents = initial_rx.borrow().clone();

    let initial_event = {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let event_data = AppEvent::FullList {
            torrents: initial_torrents,
            timestamp,
        };

        match serde_json::to_string(&event_data) {
            Ok(json) => Event::default().data(json),
            Err(_) => Event::default().comment("init_error"),
        }
    };

    // Stream that yields the initial event once
    let initial_stream = stream::once(async { Ok::<Event, Infallible>(initial_event) });

    // Stream that waits for subsequent changes via Broadcast channel
    let rx = state.event_bus.subscribe();
    let update_stream = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(event) => match serde_json::to_string(&event) {
                Ok(json) => Some((Ok::<Event, Infallible>(Event::default().data(json)), rx)),
                Err(e) => {
                    tracing::warn!("Failed to serialize SSE event: {}", e);
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

    Sse::new(initial_stream.chain(update_stream))
        .keep_alive(axum::response::sse::KeepAlive::default())
}
