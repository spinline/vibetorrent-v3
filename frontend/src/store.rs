use futures::StreamExt;
use gloo_net::eventsource::futures::EventSource;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{AppEvent, GlobalStats, NotificationLevel, Torrent};
use std::collections::HashMap;
use struct_patch::traits::Patch;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as BASE64_URL};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use wasm_bindgen::JsCast;

use crate::components::ui::toast::{ToastType, toast};

pub fn show_toast(level: NotificationLevel, message: impl Into<String>) {
    let msg = message.into();
    gloo_console::log!("TOAST CALL:", &msg, format!("{:?}", level));
    log::info!("Displaying toast: [{:?}] {}", level, msg);
    
    let variant = match level {
        NotificationLevel::Success => ToastType::Success,
        NotificationLevel::Error => ToastType::Error,
        NotificationLevel::Warning => ToastType::Warning,
        NotificationLevel::Info => ToastType::Info,
    };
    
    toast(msg, variant);
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
    pub user: RwSignal<Option<String>>,
    pub selected_torrent: RwSignal<Option<String>>,
    pub push_enabled: RwSignal<bool>,
}

pub fn provide_torrent_store() {
    let torrents = RwSignal::new(HashMap::new());
    let filter = RwSignal::new(FilterStatus::All);
    let search_query = RwSignal::new(String::new());
    let global_stats = RwSignal::new(GlobalStats::default());
    let user = RwSignal::new(Option::<String>::None);
    let selected_torrent = RwSignal::new(Option::<String>::None);
    let push_enabled = RwSignal::new(false);

    let show_browser_notification = crate::utils::notification::use_app_notification();

    let store = TorrentStore { torrents, filter, search_query, global_stats, user, selected_torrent, push_enabled };
    provide_context(store);

    // Initial check for push status
    spawn_local(async move {
        if let Ok(enabled) = is_push_subscribed().await {
            push_enabled.set(enabled);
        }
    });

    let global_stats_for_sse = global_stats;
    let torrents_for_sse = torrents;
    let show_browser_notification = show_browser_notification.clone();

    spawn_local(async move {
        let mut backoff_ms: u32 = 1000;
        let mut was_connected = false;
        let mut disconnect_notified = false;

        loop {
            let es_result = EventSource::new("/api/events");
            match es_result {
                Ok(mut es) => {
                    if let Ok(mut stream) = es.subscribe("message") {
                        let mut got_first_message = false;
                        while let Some(Ok((_, msg))) = stream.next().await {
                            if !got_first_message {
                                got_first_message = true;
                                backoff_ms = 1000;
                                if was_connected && disconnect_notified {
                                    show_toast(NotificationLevel::Success, "Sunucu bağlantısı yeniden kuruldu");
                                    disconnect_notified = false;
                                }
                                was_connected = true;
                            }

                            if let Some(data_str) = msg.data().as_string() {
                                if let Ok(bytes) = BASE64.decode(&data_str) {
                                    if let Ok(event) = rmp_serde::from_slice::<AppEvent>(&bytes) {
                                        match event {
                                            AppEvent::FullList(list, _) => {
                                                torrents_for_sse.update(|map| {
                                                    let new_hashes: std::collections::HashSet<String> = list.iter().map(|t| t.hash.clone()).collect();
                                                    map.retain(|hash, _| new_hashes.contains(hash));
                                                    for new_torrent in list { map.insert(new_torrent.hash.clone(), new_torrent); }
                                                });
                                            }
                                            AppEvent::Update(patch) => {
                                                if let Some(hash) = patch.hash.clone() {
                                                    torrents_for_sse.update(|map| { if let Some(t) = map.get_mut(&hash) { t.apply(patch); } });
                                                }
                                            }
                                            AppEvent::Stats(stats) => { global_stats_for_sse.set(stats); }
                                            AppEvent::Notification(n) => {
                                                show_toast(n.level.clone(), n.message.clone());
                                                if n.message.contains("tamamlandı") || n.level == shared::NotificationLevel::Error {
                                                    show_browser_notification("VibeTorrent", &n.message);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if was_connected && !disconnect_notified {
                            show_toast(NotificationLevel::Warning, "Sunucu bağlantısı kesildi, yeniden bağlanılıyor...");
                            disconnect_notified = true;
                        }
                    }
                }
                Err(_) => {
                    if was_connected && !disconnect_notified {
                        show_toast(NotificationLevel::Warning, "Sunucu bağlantısı kurulamıyor...");
                        disconnect_notified = true;
                    }
                }
            }
            gloo_timers::future::TimeoutFuture::new(backoff_ms).await;
            backoff_ms = std::cmp::min(backoff_ms * 2, 30000);
        }
    });
}

pub async fn is_push_subscribed() -> Result<bool, String> {
    let window = web_sys::window().ok_or("no window")?;
    let navigator = window.navigator();
    let sw_container = navigator.service_worker();
    
    let registration = wasm_bindgen_futures::JsFuture::from(sw_container.ready().map_err(|e| format!("{:?}", e))?)
        .await
        .map_err(|e| format!("{:?}", e))?
        .dyn_into::<web_sys::ServiceWorkerRegistration>()
        .map_err(|_| "not a registration")?;

    let push_manager = registration.push_manager();
    let subscription = wasm_bindgen_futures::JsFuture::from(push_manager.get_subscription().map_err(|e| format!("{:?}", e))?)
        .await
        .map_err(|e| format!("{:?}", e))?;

    Ok(!subscription.is_null())
}

pub async fn subscribe_to_push_notifications() {
    let window = web_sys::window().expect("no window");
    let navigator = window.navigator();
    let sw_container = navigator.service_worker();

    let registration = match wasm_bindgen_futures::JsFuture::from(sw_container.ready().expect("sw not ready")).await {
        Ok(reg) => reg.dyn_into::<web_sys::ServiceWorkerRegistration>().expect("not a reg"),
        Err(e) => { log::error!("SW Ready Error: {:?}", e); return; }
    };

    // 1. Get Public Key from Backend
    let public_key = match shared::server_fns::push::get_push_public_key().await {
        Ok(key) => key,
        Err(e) => { log::error!("Failed to get public key: {:?}", e); return; }
    };

    // 2. Convert base64 key to Uint8Array
    let decoded_key = BASE64_URL.decode(public_key.trim()).expect("invalid public key");
    let key_array = js_sys::Uint8Array::from(&decoded_key[..]);

    // 3. Prepare Options
    let mut options = web_sys::PushSubscriptionOptionsInit::new();
    options.user_visible_only(true);
    options.application_server_key(Some(&key_array.into()));

    // 4. Subscribe
    let push_manager = registration.push_manager();
    match wasm_bindgen_futures::JsFuture::from(push_manager.subscribe_with_options(&options).expect("subscribe failed")).await {
        Ok(subscription) => {
            let sub = subscription.dyn_into::<web_sys::PushSubscription>().expect("not a sub");
            let json = sub.to_json().expect("sub to json failed");
            
            // Extract keys from JSON
            let sub_obj: serde_json::Value = serde_wasm_bindgen::from_value(json).expect("serde from value failed");
            
            let endpoint = sub_obj["endpoint"].as_str().expect("no endpoint").to_string();
            let p256dh = sub_obj["keys"]["p256dh"].as_str().expect("no p256dh").to_string();
            let auth = sub_obj["keys"]["auth"].as_str().expect("no auth").to_string();

            // 5. Save to Backend
            match shared::server_fns::push::save_push_subscription(endpoint, p256dh, auth).await {
                Ok(_) => {
                    log::info!("Push subscription saved successfully");
                    toast_success("Bildirimler aktif edildi");
                }
                Err(e) => log::error!("Failed to save subscription: {:?}", e),
            }
        }
        Err(e) => log::error!("Subscription Error: {:?}", e),
    }
}

pub async fn unsubscribe_from_push_notifications() {
    let window = web_sys::window().expect("no window");
    let sw_container = window.navigator().service_worker();
    
    let registration = wasm_bindgen_futures::JsFuture::from(sw_container.ready().expect("sw not ready")).await
        .unwrap().dyn_into::<web_sys::ServiceWorkerRegistration>().unwrap();

    let push_manager = registration.push_manager();
    if let Ok(sub_future) = push_manager.get_subscription() {
        if let Ok(subscription) = wasm_bindgen_futures::JsFuture::from(sub_future).await {
            if !subscription.is_null() {
                let sub = subscription.dyn_into::<web_sys::PushSubscription>().unwrap();
                let endpoint = sub.endpoint();
                
                // 1. Unsubscribe in Browser
                let _ = wasm_bindgen_futures::JsFuture::from(sub.unsubscribe().unwrap()).await;
                
                // 2. Remove from Backend
                let _ = shared::server_fns::push::remove_push_subscription(endpoint).await;
                log::info!("Push subscription removed");
                show_toast(NotificationLevel::Info, "Bildirimler kapatıldı");
            }
        }
    }
}
