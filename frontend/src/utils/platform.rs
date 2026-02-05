/// Platform detection utilities

/// Check if running on iOS
pub fn is_ios() -> bool {
    let window = web_sys::window().expect("window should exist");
    let navigator = window.navigator();
    let user_agent = navigator.user_agent().unwrap_or_default();
    
    user_agent.contains("iPhone") 
        || user_agent.contains("iPad") 
        || user_agent.contains("iPod")
}

/// Check if running as a standalone PWA (installed on home screen)
pub fn is_standalone() -> bool {
    let window = web_sys::window().expect("window should exist");
    let navigator = window.navigator();
    
    // Check for iOS standalone mode
    if let Ok(standalone) = js_sys::Reflect::get(&navigator, &"standalone".into()) {
        if let Some(is_standalone) = standalone.as_bool() {
            return is_standalone;
        }
    }
    
    // Check display-mode via matchMedia
    if let Ok(match_media_fn) = js_sys::Reflect::get(&window, &"matchMedia".into()) {
        if match_media_fn.is_function() {
            let match_media = js_sys::Function::from(match_media_fn);
            if let Ok(result) = match_media.call1(&window, &"(display-mode: standalone)".into()) {
                if let Ok(matches) = js_sys::Reflect::get(&result, &"matches".into()) {
                    if let Some(is_match) = matches.as_bool() {
                        return is_match;
                    }
                }
            }
        }
    }
    
    false
}

/// Check if push notifications are supported
/// Only iOS Safari supports Web Push Notifications (iOS 16.4+)
/// macOS Safari does NOT support Web Push Notifications
pub fn supports_push_notifications() -> bool {
    // Only iOS supports web push, not macOS Safari
    if !is_ios() {
        return false;
    }
    
    let window = web_sys::window().expect("window should exist");
    
    // Check if Notification API exists
    if let Ok(notification_class) = js_sys::Reflect::get(&window, &"Notification".into()) {
        if notification_class.is_undefined() {
            return false;
        }
    } else {
        return false;
    }
    
    // For iOS, we'll attempt subscription which will check for PushManager
    // If it's not available, the subscription will fail gracefully
    true
}

/// Get platform-specific notification message
pub fn get_ios_notification_info() -> Option<String> {
    if is_ios() && !is_standalone() {
        Some(
            "iOS'ta push notification alabilmek için uygulamayı Ana Ekran'a eklemelisiniz. \
            Safari menüsünden 'Ana Ekrana Ekle' seçeneğini kullanın."
            .to_string()
        )
    } else {
        None
    }
}
