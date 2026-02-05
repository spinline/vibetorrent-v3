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

/// Check if running on Safari browser
pub fn is_safari() -> bool {
    let window = web_sys::window().expect("window should exist");
    let navigator = window.navigator();
    let user_agent = navigator.user_agent().unwrap_or_default();
    
    // Safari has 'Safari' in UA but Chrome/Edge also have it, so check for Chrome/Chromium absence
    user_agent.contains("Safari") && !user_agent.contains("Chrome") && !user_agent.contains("Chromium")
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
/// - iOS Safari: Supports Web Push (iOS 16.4+) only in standalone/PWA mode
/// - macOS Safari: Does NOT support Web Push
/// - Chrome/Firefox/Edge: Support Web Push on all platforms
pub fn supports_push_notifications() -> bool {
    let window = web_sys::window().expect("window should exist");
    
    // If Safari, only iOS Safari supports it (macOS Safari does not)
    if is_safari() {
        if !is_ios() {
            // macOS Safari does not support Web Push
            return false;
        }
        // iOS Safari supports it, will be checked further in app
        return true;
    }
    
    // For non-Safari browsers (Chrome, Firefox, Edge), check for PushManager
    if let Ok(navigator) = js_sys::Reflect::get(&window, &"navigator".into()) {
        if let Ok(service_worker) = js_sys::Reflect::get(&navigator, &"serviceWorker".into()) {
            if !service_worker.is_undefined() {
                // ServiceWorker exists, assume PushManager support
                // The actual PushManager check will happen during subscription
                return true;
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
