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
        let torrent_name = params.name.clone();
        
        tokio::spawn(async move {
            tracing::info!("Attempting to send Web Push notification for torrent: {}", torrent_name);
            match crate::push::send_push_notification(&push_store, &title, &body).await {
                Ok(_) => tracing::info!("Web Push notification sent successfully for: {}", torrent_name),
                Err(e) => tracing::error!("Failed to send Web Push notification for {}: {:?}", torrent_name, e),
            }
        });
    }

    StatusCode::OK
}
