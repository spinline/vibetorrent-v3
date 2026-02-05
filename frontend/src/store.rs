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
                                                    // Show toast notification
                                                    show_toast_with_signal(notifications, n.level.clone(), n.message.clone());
                                                    
                                                    // Show browser notification for critical events
                                                    let is_critical = n.message.contains("tamamlandı") 
                                                        || n.message.contains("Reconnected")
                                                        || n.message.contains("yeniden kuruldu")
                                                        || n.message.contains("Lost connection")
                                                        || n.level == shared::NotificationLevel::Error;
                                                    
                                                    if is_critical {
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
    if let Ok(notification_class) = js_sys::Reflect::get(&window, &"Notification".into()) {
        if !notification_class.is_undefined() {
            if let Ok(request_fn) = js_sys::Reflect::get(&notification_class, &"requestPermission".into()) {
                if request_fn.is_function() {
                    let request_fn_typed = js_sys::Function::from(request_fn);
                    match request_fn_typed.call0(&notification_class) {
                        Ok(promise_val) => {
                            let request_future = wasm_bindgen_futures::JsFuture::from(
                                web_sys::js_sys::Promise::from(promise_val)
                            );
                            match request_future.await {
                                Ok(_) => {
                                    log::info!("Notification permission requested");
                                }
                                Err(e) => {
                                    log::warn!("Failed to request notification permission: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to call requestPermission: {:?}", e);
                        }
                    }
                }
            }
        }
    }
    
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
    let window = web_sys::window().expect("window should exist");
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
    
    // Get subscription JSON using toJSON() method
    let json_result = match js_sys::Reflect::get(&push_subscription, &"toJSON".into()) {
        Ok(func) if func.is_function() => {
            let json_func = js_sys::Function::from(func);
            match json_func.call0(&push_subscription) {
                Ok(result) => result,
                Err(e) => {
                    log::error!("Failed to call toJSON: {:?}", e);
                    return;
                }
            }
        }
        _ => {
            log::error!("toJSON method not found on PushSubscription");
            return;
        }
    };
    
    let json_value = match js_sys::JSON::stringify(&json_result) {
        Ok(val) => val,
        Err(e) => {
            log::error!("Failed to stringify subscription: {:?}", e);
            return;
        }
    };
    
    let subscription_json_str = json_value.as_string().expect("should be string");
    
    log::info!("Push subscription: {}", subscription_json_str);
    
    // Parse and send to backend
    let subscription_data: serde_json::Value = match serde_json::from_str(&subscription_json_str) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse subscription JSON: {:?}", e);
            return;
        }
    };
    
    // Extract endpoint and keys
    let endpoint = subscription_data
        .get("endpoint")
        .and_then(|v| v.as_str())
        .expect("endpoint should exist")
        .to_string();
    
    let keys_obj = subscription_data
        .get("keys")
        .expect("keys should exist");
    
    let p256dh = keys_obj
        .get("p256dh")
        .and_then(|v| v.as_str())
        .expect("p256dh should exist")
        .to_string();
    
    let auth = keys_obj
        .get("auth")
        .and_then(|v| v.as_str())
        .expect("auth should exist")
        .to_string();
    
    let push_data = PushSubscriptionData {
        endpoint,
        keys: PushKeys { p256dh, auth },
    };
    
    // Send to backend
    let response = match Request::post("/api/push/subscribe")
        .json(&push_data)
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
fn url_base64_to_uint8array(base64_string: &str) -> Result<js_sys::Uint8Array, JsValue> {
    // Add padding
    let padding = (4 - (base64_string.len() % 4)) % 4;
    let mut padded = base64_string.to_string();
    padded.push_str(&"=".repeat(padding));
    
    // Replace URL-safe characters
    let standard_base64 = padded.replace('-', "+").replace('_', "/");
    
    // Decode base64
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let decoded = window.atob(&standard_base64)?;
    
    // Convert to Uint8Array
    let array = js_sys::Uint8Array::new_with_length(decoded.len() as u32);
    for (i, byte) in decoded.bytes().enumerate() {
        array.set_index(i as u32, byte);
    }
    
    Ok(array)
}
