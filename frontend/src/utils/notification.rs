use wasm_bindgen::prelude::*;
use web_sys::{Notification, NotificationOptions};
use leptos::prelude::*;
use reactive_graph::traits::Get; // Signal::get() için gerekli
use leptos_use::{use_web_notification, UseWebNotificationReturn, NotificationPermission};

/// Request browser notification permission from user
pub async fn request_notification_permission() -> bool {
    if let Ok(promise) = Notification::request_permission() {
        if let Ok(result) = wasm_bindgen_futures::JsFuture::from(promise).await {
            return result.as_string().unwrap_or_default() == "granted";
        }
    }
    false
}

/// Check if browser notifications are supported
pub fn is_notification_supported() -> bool {
    let window = web_sys::window().expect("no global window");
    js_sys::Reflect::has(&window, &JsValue::from_str("Notification")).unwrap_or(false)
}

/// Get current notification permission status
pub fn get_notification_permission() -> String {
    match Notification::permission() {
        web_sys::NotificationPermission::Granted => "granted".to_string(),
        web_sys::NotificationPermission::Denied => "denied".to_string(),
        web_sys::NotificationPermission::Default => "default".to_string(),
        _ => "default".to_string(),
    }
}

/// Hook for using browser notifications within Leptos components or effects.
/// This uses leptos-use for reactive permission tracking.
pub fn use_app_notification() -> impl Fn(&str, &str) + Clone {
    let UseWebNotificationReturn { permission, .. } = use_web_notification();
    
    move |title: &str, body: &str| {
        // Check user preference from localStorage
        let window = web_sys::window().expect("no global window");
        let storage = window.local_storage().ok().flatten();
        let enabled = storage
            .and_then(|s| s.get_item("vibetorrent_browser_notifications").ok().flatten())
            .unwrap_or_else(|| "true".to_string());

        // Use the reactive permission signal from leptos-use
        if enabled == "true" && permission.get() == NotificationPermission::Granted {
            show_browser_notification(title, body);
        }
    }
}

/// Show a browser notification (non-reactive version)
pub fn show_browser_notification(title: &str, body: &str) -> bool {
    if get_notification_permission() != "granted" {
        return false;
    }
    
    let opts = NotificationOptions::new();
    opts.set_body(body);
    opts.set_icon("/icon-192.png");
    opts.set_tag("vibetorrent");
    
    match Notification::new_with_options(title, &opts) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Legacy helper for showing notification if enabled in settings
pub fn show_notification_if_enabled(title: &str, body: &str) -> bool {
    let window = web_sys::window().expect("no global window");
    let storage = window.local_storage().ok().flatten();
    let enabled = storage
        .and_then(|s| s.get_item("vibetorrent_browser_notifications").ok().flatten())
        .unwrap_or_else(|| "true".to_string());
        
    if enabled == "true" {
        return show_browser_notification(title, body);
    }
    
    false
}