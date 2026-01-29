use axum::response::sse::{Event, Sse};
use futures::stream::{self, Stream};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;
use crate::models::{AppEvent, Torrent};
use crate::xmlrpc::{RtorrentClient, parse_multicall_response};

// Helper (should be moved to utils)
fn parse_size(s: &str) -> i64 {
    s.parse().unwrap_or(0)
}

fn parse_float(s: &str) -> f64 {
    // rTorrent usually returns integers for bytes done etc. 
    // We might need to handle empty strings.
    s.parse().unwrap_or(0.0)
}

pub async fn fetch_torrents(client: &RtorrentClient) -> Result<Vec<Torrent>, String> {
    // d.multicall2("", "main", ...)
    let params = vec![
        "", 
        "main", 
        "d.hash=", 
        "d.name=", 
        "d.size_bytes=", 
        "d.bytes_done=", 
        "d.down.rate=", 
        "d.up.rate=",
        "d.state=",          // 6
        "d.complete=",       // 7
        "d.message=",        // 8
        "d.left_bytes=",     // 9
        "d.creation_date=",  // 10
        "d.hashing=",        // 11
    ];

    match client.call("d.multicall2", &params).await {
        Ok(xml) => {
            if xml.trim().is_empty() {
                return Err("Empty response from SCGI".to_string());
            }
            match parse_multicall_response(&xml) {
                Ok(rows) => {
                    let torrents = rows.into_iter().map(|row| {
                        // row map indexes:
                        // 0: hash, 1: name, 2: size, 3: completed, 4: down_rate, 5: up_rate
                        // 6: state, 7: complete, 8: message, 9: left_bytes, 10: added, 11: hashing
                        
                        let hash = row.get(0).cloned().unwrap_or_default();
                        let name = row.get(1).cloned().unwrap_or_default();
                        let size = parse_size(row.get(2).unwrap_or(&"0".to_string()));
                        let completed = parse_size(row.get(3).unwrap_or(&"0".to_string()));
                        let down_rate = parse_size(row.get(4).unwrap_or(&"0".to_string()));
                        let up_rate = parse_size(row.get(5).unwrap_or(&"0".to_string()));
                        
                        let state = parse_size(row.get(6).unwrap_or(&"0".to_string()));
                        let is_complete = parse_size(row.get(7).unwrap_or(&"0".to_string()));
                        let message = row.get(8).cloned().unwrap_or_default();
                        let left_bytes = parse_size(row.get(9).unwrap_or(&"0".to_string()));
                        let added_date = parse_size(row.get(10).unwrap_or(&"0".to_string()));
                        let is_hashing = parse_size(row.get(11).unwrap_or(&"0".to_string()));

                        let percent_complete = if size > 0 {
                            (completed as f64 / size as f64) * 100.0
                        } else {
                            0.0
                        };
                        
                        // Status Logic
                        let status = if !message.is_empty() {
                            crate::models::TorrentStatus::Error
                        } else if is_hashing != 0 {
                            crate::models::TorrentStatus::Checking
                        } else if state == 0 {
                            crate::models::TorrentStatus::Paused
                        } else if is_complete != 0 {
                            crate::models::TorrentStatus::Seeding
                        } else {
                            crate::models::TorrentStatus::Downloading
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
                        }
                    }).collect();
                    Ok(torrents)
                },
                Err(e) => {
                    Err(format!("XML Parse Error: {}", e))
                }
            }
        },
        Err(e) => {
            Err(format!("RPC Error: {}", e))
        }
    }
}

use axum::extract::State;
use crate::models::AppState;

pub async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Get initial value synchronously (from the watch channel's current state)
    let initial_rx = state.tx.subscribe();
    let initial_torrents = initial_rx.borrow().clone();
    
    let initial_event = {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let event_data = AppEvent::FullList(initial_torrents, timestamp);
        match serde_json::to_string(&event_data) {
             Ok(json) => Event::default().data(json),
             Err(_) => Event::default().comment("init_error"),
        }
    };
    
    // Stream that yields the initial event once
    let initial_stream = stream::once(async { Ok::<Event, Infallible>(initial_event) });
    
    // Stream that waits for subsequent changes
    let update_stream = stream::unfold(state.tx.subscribe(), |mut rx| async move {
         if let Err(_) = rx.changed().await {
            return None;
        }
        let torrents = rx.borrow().clone();
        // println!("Broadcasting SSE update with {} items", torrents.len());
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let event_data = AppEvent::FullList(torrents, timestamp);
        
         match serde_json::to_string(&event_data) {
            Ok(json) => Some((Ok::<Event, Infallible>(Event::default().data(json)), rx)),
            Err(_) => Some((Ok::<Event, Infallible>(Event::default().comment("error")), rx)),
         }
    });
    
    Sse::new(initial_stream.chain(update_stream))
        .keep_alive(axum::response::sse::KeepAlive::default())
}
