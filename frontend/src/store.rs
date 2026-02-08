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
    pub user: RwSignal<Option<String>>,
}

pub fn provide_torrent_store() {
    let torrents = create_rw_signal(Vec::<Torrent>::new());
    let filter = create_rw_signal(FilterStatus::All);
    let search_query = create_rw_signal(String::new());
    let global_stats = create_rw_signal(GlobalStats::default());
    let notifications = create_rw_signal(Vec::<NotificationItem>::new());
    let user = create_rw_signal(Option::<String>::None);

    // Browser notification hook
    let show_browser_notification = crate::utils::notification::use_app_notification();

    let store = TorrentStore {
        torrents,
        filter,
        search_query,
        global_stats,
        notifications,
        user,
    };
    provide_context(store);

    // Initialize SSE connection with auto-reconnect
    create_effect(move |_| {
        // Sadece kullanıcı giriş yapmışsa bağlantıyı başlat
        if user.get().is_none() {
            logging::log!("SSE: User not authenticated, skipping connection.");
            return;
        }

        let show_browser_notification = show_browser_notification.clone();
        spawn_local(async move {
            let mut backoff_ms: u32 = 1000; // Start with 1 second
            let max_backoff_ms: u32 = 30000; // Max 30 seconds
            let mut was_connected = false;
            let mut disconnect_notified = false; // Track if we already showed disconnect toast
            let mut got_first_message; // Only count as "connected" after receiving data

            loop {
                let es_result = EventSource::new("/api/events");

                match es_result {
                    Ok(mut es) => {
                        match es.subscribe("message") {
                            Ok(mut stream) => {
                                // Don't show "connected" toast yet - wait for first real message
                                got_first_message = false;

                                // Process messages
                                while let Some(Ok((_, msg))) = stream.next().await {
                                    // First successful message = truly connected
                                    if !got_first_message {
                                        got_first_message = true;
                                        backoff_ms = 1000; // Reset backoff on real data

                                        if was_connected && disconnect_notified {
                                            // We were previously connected, lost connection, and now truly reconnected
                                            show_toast_with_signal(
                                                notifications,
                                                NotificationLevel::Success,
                                                "Sunucu bağlantısı yeniden kuruldu",
                                            );
                                            disconnect_notified = false;
                                        }
                                        was_connected = true;
                                    }

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
                                                    // Show toast notification
                                                    show_toast_with_signal(notifications, n.level.clone(), n.message.clone());

                                                    // Show browser notification for critical events
                                                    let is_critical = n.message.contains("tamamlandı")
                                                        || n.level == shared::NotificationLevel::Error;

                                                    if is_critical {
                                                        let title = match n.level {
                                                            shared::NotificationLevel::Success => "✅ VibeTorrent",
                                                            shared::NotificationLevel::Error => "❌ VibeTorrent",
                                                            shared::NotificationLevel::Warning => "⚠️ VibeTorrent",
                                                            shared::NotificationLevel::Info => "ℹ️ VibeTorrent",
                                                        };

                                                        show_browser_notification(
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
                                if was_connected && !disconnect_notified {
                                    show_toast_with_signal(
                                        notifications,
                                        NotificationLevel::Warning,
                                        "Sunucu bağlantısı kesildi, yeniden bağlanılıyor...",
                                    );
                                    disconnect_notified = true;
                                }
                            }
                            Err(_) => {
                                // Failed to subscribe - only notify once
                                if was_connected && !disconnect_notified {
                                    show_toast_with_signal(
                                        notifications,
                                        NotificationLevel::Warning,
                                        "Sunucu bağlantısı kesildi, yeniden bağlanılıyor...",
                                    );
                                    disconnect_notified = true;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Failed to create EventSource - only notify once
                        if was_connected && !disconnect_notified {
                            show_toast_with_signal(
                                notifications,
                                NotificationLevel::Warning,
                                "Sunucu bağlantısı kesildi, yeniden bağlanılıyor...",
                            );
                            disconnect_notified = true;
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

// ============================================================================
// Push Notification Subscription
// ============================================================================

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct PushSubscriptionData {
    endpoint: String,
    keys: PushKeys,
}

#[derive(Serialize, Deserialize, Debug)]
struct PushKeys {
    p256dh: String,
    auth: String,
}

/// Subscribe user to push notifications
/// Requests notification permission if needed, then subscribes to push
pub async fn subscribe_to_push_notifications() {
    use gloo_net::http::Request;

    // First, request notification permission if not already granted
    let window = web_sys::window().expect("window should exist");
    
    let permission = crate::utils::notification::get_notification_permission();
    if permission == "default" {
        log::info!("Requesting notification permission...");
        if !crate::utils::notification::request_notification_permission().await {
            log::warn!("Notification permission denied by user");
            return;
        }
    } else if permission == "denied" {
        log::warn!("Notification permission was denied");
        return;
    }

    log::info!("Notification permission granted! Proceeding with push subscription...");


    // Get VAPID public key from backend
    let public_key_response = match Request::get("/api/push/public-key").send().await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to get VAPID public key: {:?}", e);
            return;
        }
    };

    let public_key_data: serde_json::Value = match public_key_response.json().await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse VAPID public key: {:?}", e);
            return;
        }
    };

    let public_key = match public_key_data.get("publicKey").and_then(|v| v.as_str()) {
        Some(key) => key,
        None => {
            log::error!("Missing publicKey in response");
            return;
        }
    };

    log::info!("VAPID public key from backend: {} (len: {})", public_key, public_key.len());

    // Convert VAPID public key to Uint8Array
    let public_key_array = match url_base64_to_uint8array(public_key) {
        Ok(arr) => {
            log::info!("VAPID key converted to Uint8Array (length: {})", arr.length());
            arr
        }
        Err(e) => {
            log::error!("Failed to convert VAPID key: {:?}", e);
            return;
        }
    };

    // Get service worker registration
    let navigator = window.navigator();
    let service_worker = navigator.service_worker();

    let registration_promise = match service_worker.ready() {
        Ok(promise) => promise,
        Err(e) => {
            log::error!("Failed to get ready promise: {:?}", e);
            return;
        }
    };

    let registration_future = wasm_bindgen_futures::JsFuture::from(registration_promise);

    let registration = match registration_future.await {
        Ok(reg) => reg,
        Err(e) => {
            log::error!("Failed to get service worker registration: {:?}", e);
            return;
        }
    };

    let service_worker_registration = registration
        .dyn_into::<web_sys::ServiceWorkerRegistration>()
        .expect("should be ServiceWorkerRegistration");

    // Subscribe to push
    let push_manager = match service_worker_registration.push_manager() {
        Ok(pm) => pm,
        Err(e) => {
            log::error!("Failed to get push manager: {:?}", e);
            return;
        }
    };

    let subscribe_options = web_sys::PushSubscriptionOptionsInit::new();
    subscribe_options.set_user_visible_only(true);
    subscribe_options.set_application_server_key(&public_key_array);

    let subscribe_promise = match push_manager.subscribe_with_options(&subscribe_options) {
        Ok(promise) => promise,
        Err(e) => {
            log::error!("Failed to subscribe to push: {:?}", e);
            return;
        }
    };

    let subscription_future = wasm_bindgen_futures::JsFuture::from(subscribe_promise);

    let subscription = match subscription_future.await {
        Ok(sub) => sub,
        Err(e) => {
            log::error!("Failed to get push subscription: {:?}", e);
            return;
        }
    };

    let push_subscription = subscription
        .dyn_into::<web_sys::PushSubscription>()
        .expect("should be PushSubscription");

    // PushSubscription objects can be serialized directly via JSON.stringify which calls their toJSON method internally.
    // Or we can use Reflect to call toJSON if we want the object directly.
    // Let's use the robust way: call toJSON via Reflect but handle it gracefully.
    let json_val = match js_sys::Reflect::get(&push_subscription, &"toJSON".into()) {
        Ok(func) if func.is_function() => {
             let json_func = js_sys::Function::from(func);
             match json_func.call0(&push_subscription) {
                 Ok(res) => res,
                 Err(e) => {
                     log::error!("Failed to call toJSON: {:?}", e);
                     return;
                 }
             }
        }
        _ => {
            // Fallback: try to stringify the object directly
            // log::warn!("toJSON not found, trying JSON.stringify");
            let json_str = match js_sys::JSON::stringify(&push_subscription) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to stringify subscription: {:?}", e);
                    return;
                }
            };
            // Parse back to object to match our expected flow (slightly inefficient but safe)
            match js_sys::JSON::parse(&String::from(json_str)) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("Failed to parse stringified subscription: {:?}", e);
                    return;
                }
            }
        }
    };
    
    // Convert JsValue (JSON object) to PushSubscriptionJSON struct via serde
    // Note: web_sys::PushSubscriptionJSON is not a struct we can directly use with serde_json usually,
    // but we can use serde-wasm-bindgen to convert JsValue -> Rust Struct
    let subscription_data: PushSubscriptionData = match serde_wasm_bindgen::from_value(json_val) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse subscription JSON: {:?}", e);
            return;
        }
    };

    // Send to backend (subscription_data is already the struct we need)
    let response = match Request::post("/api/push/subscribe")
        .json(&subscription_data)
        .expect("serialization should succeed")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to send subscription to backend: {:?}", e);
            return;
        }
    };

    if response.ok() {
        log::info!("Successfully subscribed to push notifications");
    } else {
        log::error!("Backend rejected push subscription: {:?}", response.status());
    }
}

/// Helper to convert URL-safe base64 string to Uint8Array
/// Uses pure Rust base64 crate for better safety and performance
fn url_base64_to_uint8array(base64_string: &str) -> Result<js_sys::Uint8Array, JsValue> {
    use base64::{engine::general_purpose, Engine as _};

    // VAPID keys are URL-safe base64. Try both NO_PAD and padded for robustness.
    let bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(base64_string)
        .or_else(|_| general_purpose::URL_SAFE.decode(base64_string))
        .map_err(|e| JsValue::from_str(&format!("Base64 decode error: {}", e)))?;

    Ok(js_sys::Uint8Array::from(&bytes[..]))
}
