use futures::StreamExt;
use gloo_net::eventsource::futures::EventSource;
use leptos::prelude::*;
use shared::{AppEvent, GlobalStats, NotificationLevel, SystemNotification, Torrent};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq)]
pub struct NotificationItem {
    pub id: u64,
    pub notification: SystemNotification,
}

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

    leptos::prelude::set_timeout(
        move || {
            notifications.update(|list| list.retain(|i| i.id != id));
        },
        std::time::Duration::from_secs(5),
    );
}

pub fn show_toast(level: NotificationLevel, message: impl Into<String>) {
    if let Some(store) = use_context::<TorrentStore>() {
        show_toast_with_signal(store.notifications, level, message);
    }
}

pub fn toast_success(message: impl Into<String>) { show_toast(NotificationLevel::Success, message); }
pub fn toast_error(message: impl Into<String>) { show_toast(NotificationLevel::Error, message); }

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PushSubscriptionData {
    pub endpoint: String,
    pub keys: PushKeys,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterStatus {
    All, Downloading, Seeding, Completed, Paused, Inactive, Active, Error,
}

#[derive(Clone, Copy, Debug)]
pub struct TorrentStore {
    pub torrents: RwSignal<HashMap<String, Torrent>>,
    pub filter: RwSignal<FilterStatus>,
    pub search_query: RwSignal<String>,
    pub global_stats: RwSignal<GlobalStats>,
    pub notifications: RwSignal<Vec<NotificationItem>>,
    pub user: RwSignal<Option<String>>,
}

pub fn provide_torrent_store() {
    let torrents = RwSignal::new(HashMap::new());
    let filter = RwSignal::new(FilterStatus::All);
    let search_query = RwSignal::new(String::new());
    let global_stats = RwSignal::new(GlobalStats::default());
    let notifications = RwSignal::new(Vec::<NotificationItem>::new());
    let user = RwSignal::new(Option::<String>::None);

    let show_browser_notification = crate::utils::notification::use_app_notification();

    let store = TorrentStore { torrents, filter, search_query, global_stats, notifications, user };
    provide_context(store);

    let user_for_sse = user;
    let notifications_for_sse = notifications;
    let global_stats_for_sse = global_stats;
    let torrents_for_sse = torrents;
    let show_browser_notification = show_browser_notification.clone();

    spawn_local(async move {
        let mut backoff_ms: u32 = 1000;
        let mut was_connected = false;
        let mut disconnect_notified = false;

        loop {
            let user_val = user_for_sse.get();
            if user_val.is_none() {
                log::debug!("SSE: User not authenticated, waiting...");
                gloo_timers::future::TimeoutFuture::new(1000).await;
                continue;
            }

            log::debug!("SSE: Creating EventSource...");
            let es_result = EventSource::new("/api/events");
            match es_result {
                Ok(mut es) => {
                    log::debug!("SSE: EventSource created, subscribing...");
                    if let Ok(mut stream) = es.subscribe("message") {
                        log::debug!("SSE: Subscribed to message channel");
                        let mut got_first_message = false;
                        while let Some(Ok((_, msg))) = stream.next().await {
                            log::debug!("SSE: Received message");
                            if !got_first_message {
                                got_first_message = true;
                                backoff_ms = 1000;
                                if was_connected && disconnect_notified {
                                    show_toast_with_signal(notifications_for_sse, NotificationLevel::Success, "Sunucu bağlantısı yeniden kuruldu");
                                    disconnect_notified = false;
                                }
                                was_connected = true;
                            }

                            if let Some(data_str) = msg.data().as_string() {
                                log::debug!("SSE: Parsing JSON: {}", data_str);
                                if let Ok(event) = serde_json::from_str::<AppEvent>(&data_str) {
                                    match event {
                                        AppEvent::FullList { torrents: list, .. } => {
                                            log::info!("SSE: Received FullList with {} torrents", list.len());
                                            torrents_for_sse.update(|map| {
                                                let new_hashes: std::collections::HashSet<String> = list.iter().map(|t| t.hash.clone()).collect();
                                                map.retain(|hash, _| new_hashes.contains(hash));
                                                for new_torrent in list {
                                                    map.insert(new_torrent.hash.clone(), new_torrent);
                                                }
                                            });
                                            log::debug!("SSE: torrents map now has {} entries", torrents_for_sse.with(|m| m.len()));
                                        }
                                        AppEvent::Update(update) => {
                                            torrents_for_sse.update(|map| {
                                                if let Some(t) = map.get_mut(&update.hash) {
                                                    if let Some(v) = update.name { t.name = v; }
                                                    if let Some(v) = update.size { t.size = v; }
                                                    if let Some(v) = update.down_rate { t.down_rate = v; }
                                                    if let Some(v) = update.up_rate { t.up_rate = v; }
                                                    if let Some(v) = update.percent_complete { t.percent_complete = v; }
                                                    if let Some(v) = update.completed { t.completed = v; }
                                                    if let Some(v) = update.eta { t.eta = v; }
                                                    if let Some(v) = update.status { t.status = v; }
                                                    if let Some(v) = update.error_message { t.error_message = v; }
                                                    if let Some(v) = update.label { t.label = Some(v); }
                                                }
                                            });
                                        }
                                        AppEvent::Stats(stats) => { global_stats_for_sse.set(stats); }
                                        AppEvent::Notification(n) => {
                                            show_toast_with_signal(notifications_for_sse, n.level.clone(), n.message.clone());
                                            if n.message.contains("tamamlandı") || n.level == shared::NotificationLevel::Error {
                                                show_browser_notification("VibeTorrent", &n.message);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if was_connected && !disconnect_notified {
                            show_toast_with_signal(notifications_for_sse, NotificationLevel::Warning, "Sunucu bağlantısı kesildi, yeniden bağlanılıyor...");
                            disconnect_notified = true;
                        }
                    }
                }
                Err(_) => {
                    if was_connected && !disconnect_notified {
                        show_toast_with_signal(notifications_for_sse, NotificationLevel::Warning, "Sunucu bağlantısı kurulamıyor...");
                        disconnect_notified = true;
                    }
                }
            }
            log::debug!("SSE: Reconnecting in {}ms...", backoff_ms);
            gloo_timers::future::TimeoutFuture::new(backoff_ms).await;
            backoff_ms = std::cmp::min(backoff_ms * 2, 30000);
        }
    });
}

pub async fn subscribe_to_push_notifications() {
    // ...
}
