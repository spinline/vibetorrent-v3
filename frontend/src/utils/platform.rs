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
pub fn supports_push_notifications() -> bool {
    let window = web_sys::window().expect("window should exist");
    
    // Check if PushManager exists
    if let Ok(navigator) = js_sys::Reflect::get(&window, &"navigator".into()) {
        if let Ok(service_worker) = js_sys::Reflect::get(&navigator, &"serviceWorker".into()) {
            if let Ok(push_manager) = js_sys::Reflect::get(&service_worker, &"PushManager".into()) {
                return !push_manager.is_undefined();
            }
        }
    }
    
    false
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
