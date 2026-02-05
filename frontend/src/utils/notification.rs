use wasm_bindgen::prelude::*;
use web_sys::{Notification, NotificationOptions};

/// Request browser notification permission from user
pub async fn request_notification_permission() -> bool {
    let window = web_sys::window().expect("no global window");
    
    // Check if Notification API is available
    if js_sys::Reflect::has(&window, &JsValue::from_str("Notification")).unwrap_or(false) {
        let notification = js_sys::Reflect::get(&window, &JsValue::from_str("Notification"))
            .expect("Notification should exist");
        
        // Request permission
        let promise = js_sys::Reflect::get(&notification, &JsValue::from_str("requestPermission"))
            .expect("requestPermission should exist");
        
        if let Ok(function) = promise.dyn_into::<js_sys::Function>() {
            if let Ok(promise) = function.call0(&notification) {
                if let Ok(promise) = promise.dyn_into::<js_sys::Promise>() {
                    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
                    
                    if let Ok(permission) = result {
                        let permission_str = permission.as_string().unwrap_or_default();
                        return permission_str == "granted";
                    }
                }
            }
        }
    }
    
    false
}

/// Check if browser notifications are supported and permitted
pub fn is_notification_supported() -> bool {
    let window = web_sys::window().expect("no global window");
    js_sys::Reflect::has(&window, &JsValue::from_str("Notification")).unwrap_or(false)
}

/// Get current notification permission status
pub fn get_notification_permission() -> String {
    if !is_notification_supported() {
        return "unsupported".to_string();
    }
    
    let window = web_sys::window().expect("no global window");
    let notification = js_sys::Reflect::get(&window, &JsValue::from_str("Notification"))
        .expect("Notification should exist");
    
    let permission = js_sys::Reflect::get(&notification, &JsValue::from_str("permission"))
        .unwrap_or(JsValue::from_str("default"));
    
    permission.as_string().unwrap_or("default".to_string())
}

/// Show a browser notification
/// Returns true if notification was shown successfully
pub fn show_browser_notification(title: &str, body: &str, icon: Option<&str>) -> bool {
    // Check permission first
    let permission = get_notification_permission();
    if permission != "granted" {
        log::warn!("Notification permission not granted: {}", permission);
        return false;
    }
    
    // Create notification options
    let opts = NotificationOptions::new();
    opts.set_body(body);
    opts.set_icon(icon.unwrap_or("/icon-192.png"));
    opts.set_badge("/icon-192.png");
    opts.set_tag("vibetorrent");
    opts.set_require_interaction(false);
    opts.set_silent(Some(false));
    
    // Create and show notification
    match Notification::new_with_options(title, &opts) {
        Ok(_notification) => {
            true
        }
        Err(e) => {
            log::error!("Failed to create notification: {:?}", e);
            false
        }
    }
}

/// Show notification only if enabled in settings and permission granted
pub fn show_notification_if_enabled(title: &str, body: &str) -> bool {
    // Check localStorage for user preference
    let window = web_sys::window().expect("no global window");
    let storage = window.local_storage().ok().flatten();
    
    if let Some(storage) = storage {
        let enabled = storage
            .get_item("vibetorrent_browser_notifications")
            .ok()
            .flatten()
            .unwrap_or("true".to_string());
        
        if enabled == "true" {
            return show_browser_notification(title, body, None);
        }
    }
    
    false
}
