use axum::{
    extract::{State, Query},
    http::StatusCode,
};
use serde::Deserialize;
use shared::{AppEvent, SystemNotification, NotificationLevel};
use crate::AppState;

#[derive(Deserialize)]
pub struct TorrentFinishedQuery {
    pub name: String,
    pub hash: String,
}

pub async fn torrent_finished_handler(
    State(state): State<AppState>,
    Query(params): Query<TorrentFinishedQuery>,
) -> StatusCode {
    tracing::info!("WEBHOOK: Received notification from rTorrent. Name: {:?}, Hash: {:?}", params.name, params.hash);

    let torrent_name = if params.name.is_empty() || params.name == "$d.name=" {
        "Bilinmeyen Torrent".to_string()
    } else {
        params.name.clone()
    };

    let message = format!("Torrent tamamlandı: {}", torrent_name);

    // 1. Send to active SSE clients (for Toast)
    let notification = SystemNotification {
        level: NotificationLevel::Success,
        message: message.clone(),
    };
    let _ = state.event_bus.send(AppEvent::Notification(notification));

    // 2. Send Web Push Notification (for Background)
    #[cfg(feature = "push-notifications")]
    {
        let push_store = state.push_store.clone();
        let title = "Torrent Tamamlandı".to_string();
        let body = message;
        let name_for_log = torrent_name.clone();
        
        tokio::spawn(async move {
            tracing::info!("Attempting to send Web Push notification for torrent: {}", name_for_log);
            match crate::push::send_push_notification(&push_store, &title, &body).await {
                Ok(_) => tracing::info!("Web Push notification task completed for: {}", name_for_log),
                Err(e) => tracing::error!("Failed to send Web Push notification for {}: {:?}", name_for_log, e),
            }
        });
    }

    StatusCode::OK
}
