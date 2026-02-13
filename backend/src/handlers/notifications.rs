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
    tracing::info!("Torrent finished notification received: {} ({})", params.name, params.hash);

    let message = format!("Torrent tamamlandı: {}", params.name);

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
        tokio::spawn(async move {
            if let Err(e) = crate::push::send_push_notification(&push_store, &title, &body).await {
                tracing::error!("Failed to send push notification from webhook: {}", e);
            }
        });
    }

    StatusCode::OK
}
