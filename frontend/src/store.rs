use futures::StreamExt;
use gloo_net::eventsource::futures::EventSource;
use leptos::*;
use shared::{AppEvent, GlobalStats, NotificationLevel, SystemNotification, Torrent};

#[derive(Clone, Debug, PartialEq)]
pub struct NotificationItem {
    pub id: u64,
    pub notification: SystemNotification,
}

// ============================================================================
// Toast Helper Functions (Clean Code: Single Responsibility)
// ============================================================================

/// Shows a toast notification using a direct signal reference.
/// Use this version inside async blocks (spawn_local) where use_context is unavailable.
/// Auto-removes after 5 seconds.
pub fn show_toast_with_signal(
    notifications: RwSignal<Vec<NotificationItem>>,
    level: NotificationLevel,
    message: impl Into<String>,
) {
    let id = js_sys::Date::now() as u64;
    let notification = SystemNotification {
        level,
        message: message.into(),
    };
    let item = NotificationItem { id, notification };
    
    notifications.update(|list| list.push(item));
    
    // Auto-remove after 5 seconds
    let _ = set_timeout(
        move || {
            notifications.update(|list| list.retain(|i| i.id != id));
        },
        std::time::Duration::from_secs(5),
    );
}

/// Shows a toast notification with the given level and message.
/// Only works within reactive scope (components, effects). For async, use show_toast_with_signal.
/// Auto-removes after 5 seconds.
pub fn show_toast(level: NotificationLevel, message: impl Into<String>) {
    if let Some(store) = use_context::<TorrentStore>() {
        show_toast_with_signal(store.notifications, level, message);
    }
}

/// Convenience function for success toasts (reactive scope only)
pub fn toast_success(message: impl Into<String>) {
    show_toast(NotificationLevel::Success, message);
}

/// Convenience function for error toasts (reactive scope only)
pub fn toast_error(message: impl Into<String>) {
    show_toast(NotificationLevel::Error, message);
}

/// Convenience function for info toasts (reactive scope only)
pub fn toast_info(message: impl Into<String>) {
    show_toast(NotificationLevel::Info, message);
}

/// Convenience function for warning toasts (reactive scope only)
pub fn toast_warning(message: impl Into<String>) {
    show_toast(NotificationLevel::Warning, message);
}

// ============================================================================
// Action Message Mapping (Clean Code: DRY Principle)
// ============================================================================

/// Maps torrent action strings to user-friendly Turkish messages.
/// Returns (success_message, error_message)
pub fn get_action_messages(action: &str) -> (&'static str, &'static str) {
    match action {
        "start" => ("Torrent başlatıldı", "Torrent başlatılamadı"),
        "stop" => ("Torrent durduruldu", "Torrent durdurulamadı"),
        "pause" => ("Torrent duraklatıldı", "Torrent duraklatılamadı"),
        "delete" => ("Torrent silindi", "Torrent silinemedi"),
        "delete_with_data" => ("Torrent ve verileri silindi", "Torrent silinemedi"),
        "recheck" => ("Torrent kontrol ediliyor", "Kontrol başlatılamadı"),
        _ => ("İşlem tamamlandı", "İşlem başarısız"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterStatus {
    All,
    Downloading,
    Seeding,
    Completed,
    Paused,
    Inactive,
    Active,
    Error,
}

impl FilterStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterStatus::All => "All",
            FilterStatus::Downloading => "Downloading",
            FilterStatus::Seeding => "Seeding",
            FilterStatus::Completed => "Completed",
            FilterStatus::Paused => "Paused",
            FilterStatus::Inactive => "Inactive",
            FilterStatus::Active => "Active",
            FilterStatus::Error => "Error",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TorrentStore {
    pub torrents: RwSignal<Vec<Torrent>>,
    pub filter: RwSignal<FilterStatus>,
    pub search_query: RwSignal<String>,
    pub global_stats: RwSignal<GlobalStats>,
    pub notifications: RwSignal<Vec<NotificationItem>>,
}

pub fn provide_torrent_store() {
    let torrents = create_rw_signal(Vec::<Torrent>::new());
    let filter = create_rw_signal(FilterStatus::All);
    let search_query = create_rw_signal(String::new());
    let global_stats = create_rw_signal(GlobalStats::default());
    let notifications = create_rw_signal(Vec::<NotificationItem>::new());

    let store = TorrentStore {
        torrents,
        filter,
        search_query,
        global_stats,
        notifications,
    };
    provide_context(store);

    // Initialize SSE connection with auto-reconnect
    create_effect(move |_| {
        spawn_local(async move {
            let mut backoff_ms: u32 = 1000; // Start with 1 second
            let max_backoff_ms: u32 = 30000; // Max 30 seconds
            let mut was_connected = false;

            loop {
                let es_result = EventSource::new("/api/events");
                
                match es_result {
                    Ok(mut es) => {
                        match es.subscribe("message") {
                            Ok(mut stream) => {
                                // Connection established
                                if was_connected {
                                    // We were previously connected and lost connection, now reconnected
                                    show_toast_with_signal(
                                        notifications,
                                        NotificationLevel::Success,
                                        "Sunucu bağlantısı yeniden kuruldu",
                                    );
                                }
                                was_connected = true;
                                backoff_ms = 1000; // Reset backoff on successful connection

                                // Process messages
                                while let Some(Ok((_, msg))) = stream.next().await {
                                    if let Some(data_str) = msg.data().as_string() {
                                        if let Ok(event) = serde_json::from_str::<AppEvent>(&data_str) {
                                            match event {
                                                AppEvent::FullList { torrents: list, .. } => {
                                                    torrents.set(list);
                                                }
                                                AppEvent::Update(update) => {
                                                    torrents.update(|list| {
                                                        if let Some(t) = list.iter_mut().find(|t| t.hash == update.hash)
                                                        {
                                                            if let Some(name) = update.name {
                                                                t.name = name;
                                                            }
                                                            if let Some(size) = update.size {
                                                                t.size = size;
                                                            }
                                                            if let Some(down_rate) = update.down_rate {
                                                                t.down_rate = down_rate;
                                                            }
                                                            if let Some(up_rate) = update.up_rate {
                                                                t.up_rate = up_rate;
                                                            }
                                                            if let Some(percent_complete) = update.percent_complete {
                                                                t.percent_complete = percent_complete;
                                                            }
                                                            if let Some(completed) = update.completed {
                                                                t.completed = completed;
                                                            }
                                                            if let Some(eta) = update.eta {
                                                                t.eta = eta;
                                                            }
                                                            if let Some(status) = update.status {
                                                                t.status = status;
                                                            }
                                                            if let Some(error_message) = update.error_message {
                                                                t.error_message = error_message;
                                                            }
                                                            if let Some(label) = update.label {
                                                                t.label = Some(label);
                                                            }
                                                        }
                                                    });
                                                }
                                                AppEvent::Stats(stats) => {
                                                    global_stats.set(stats);
                                                }
                                                AppEvent::Notification(n) => {
                                                    log::info!("📬 Received notification: {} - {}", n.level == shared::NotificationLevel::Success, n.message);
                                                    
                                                    // Show toast notification
                                                    show_toast_with_signal(notifications, n.level.clone(), n.message.clone());
                                                    
                                                    // Show browser notification for critical events
                                                    // (torrent completed, connection lost/restored, errors)
                                                    let is_critical = n.message.contains("tamamlandı") 
                                                        || n.message.contains("Reconnected")
                                                        || n.message.contains("yeniden kuruldu")
                                                        || n.message.contains("Lost connection")
                                                        || n.level == shared::NotificationLevel::Error;
                                                    
                                                    if is_critical {
                                                        log::info!("🔴 Critical notification detected: {}", n.message);
                                                        let title = match n.level {
                                                            shared::NotificationLevel::Success => "✅ VibeTorrent",
                                                            shared::NotificationLevel::Error => "❌ VibeTorrent",
                                                            shared::NotificationLevel::Warning => "⚠️ VibeTorrent",
                                                            shared::NotificationLevel::Info => "ℹ️ VibeTorrent",
                                                        };
                                                        
                                                        crate::utils::notification::show_notification_if_enabled(
                                                            title,
                                                            &n.message
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Stream ended - connection lost
                                if was_connected {
                                    show_toast_with_signal(
                                        notifications,
                                        NotificationLevel::Warning,
                                        "Sunucu bağlantısı kesildi, yeniden bağlanılıyor...",
                                    );
                                }
                            }
                            Err(_) => {
                                // Failed to subscribe
                                if was_connected {
                                    show_toast_with_signal(
                                        notifications,
                                        NotificationLevel::Warning,
                                        "Sunucu bağlantısı kesildi, yeniden bağlanılıyor...",
                                    );
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Failed to create EventSource
                        if was_connected {
                            show_toast_with_signal(
                                notifications,
                                NotificationLevel::Warning,
                                "Sunucu bağlantısı kesildi, yeniden bağlanılıyor...",
                            );
                        }
                    }
                }

                // Wait before reconnecting (exponential backoff)
                gloo_timers::future::TimeoutFuture::new(backoff_ms).await;
                backoff_ms = std::cmp::min(backoff_ms * 2, max_backoff_ms);
            }
        });
    });
}
