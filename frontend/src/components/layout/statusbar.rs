use leptos::*;
use shared::GlobalLimitRequest;
use wasm_bindgen::JsCast;

fn format_bytes(bytes: i64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    let i = (bytes as f64).log2().div_euclid(10.0) as usize;
    format!(
        "{:.1} {}",
        (bytes as f64) / 1024_f64.powi(i as i32),
        UNITS[i]
    )
}

fn format_speed(bytes_per_sec: i64) -> String {
    if bytes_per_sec == 0 {
        return "0 B/s".to_string();
    }
    format!("{}/s", format_bytes(bytes_per_sec))
}

#[component]
pub fn StatusBar() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let stats = store.global_stats;

    let initial_theme = if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            doc.document_element()
                .and_then(|el| el.get_attribute("data-theme"))
                .unwrap_or_else(|| "dark".to_string())
        } else {
            "dark".to_string()
        }
    } else {
        "dark".to_string()
    };

    let (current_theme, set_current_theme) = create_signal(initial_theme);

    create_effect(move |_| {
        if let Some(win) = web_sys::window() {
            if let Some(storage) = win.local_storage().ok().flatten() {
                if let Ok(Some(stored_theme)) = storage.get_item("vibetorrent_theme") {
                    let theme = stored_theme.to_lowercase();
                    set_current_theme.set(theme.clone());
                    if let Some(doc) = win.document() {
                        let _ = doc
                            .document_element()
                            .unwrap()
                            .set_attribute("data-theme", &theme);
                    }
                }
            }
        }
    });

    // Preset limits in bytes/s
    let limits: Vec<(i64, &str)> = vec![
        (0, "Unlimited"),
        (100 * 1024, "100 KB/s"),
        (500 * 1024, "500 KB/s"),
        (1024 * 1024, "1 MB/s"),
        (2 * 1024 * 1024, "2 MB/s"),
        (5 * 1024 * 1024, "5 MB/s"),
        (10 * 1024 * 1024, "10 MB/s"),
        (20 * 1024 * 1024, "20 MB/s"),
    ];

    let set_limit = move |limit_type: &str, val: i64| {
        let limit_type = limit_type.to_string();
        logging::log!("Setting {} limit to {}", limit_type, val);

        spawn_local(async move {
            let req_body = if limit_type == "down" {
                GlobalLimitRequest {
                    max_download_rate: Some(val),
                    max_upload_rate: None,
                }
            } else {
                GlobalLimitRequest {
                    max_download_rate: None,
                    max_upload_rate: Some(val),
                }
            };

            let client =
                gloo_net::http::Request::post("/api/settings/global-limits").json(&req_body);

            match client {
                Ok(req) => match req.send().await {
                    Ok(resp) => {
                        if !resp.ok() {
                            logging::error!(
                                "Failed to set limit: {} {}",
                                resp.status(),
                                resp.status_text()
                            );
                        } else {
                            logging::log!("Limit set successfully");
                        }
                    }
                    Err(e) => logging::error!("Network error setting limit: {}", e),
                },
                Err(e) => logging::error!("Failed to create request: {}", e),
            }
        });
    };

    // Helper to close dropdowns by blurring the active element
    let close_dropdown = move || {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(active) = doc.active_element() {
                let _ = active
                    .dyn_into::<web_sys::HtmlElement>()
                    .map(|el| el.blur());
            }
        }
    };

    // Global listener to force blur on touchstart (for iOS "tap outside" closing)
    let force_blur = move |_| {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(active) = doc.active_element() {
                // If something is focused, blur it to close dropdowns
                let _ = active
                    .dyn_into::<web_sys::HtmlElement>()
                    .map(|el| el.blur());
            }
        }
    };
    let _ = window_event_listener(ev::touchstart, force_blur);

    view! {
        <div class="h-8 min-h-8 bg-base-200 border-t border-base-300 flex items-center px-4 text-xs gap-4 text-base-content/70">

            // --- DOWNLOAD SPEED DROPDOWN ---
            <div
                class="dropdown dropdown-top dropdown-start"
                on:touchstart=move |e| e.stop_propagation()
            >
                <div
                    tabindex="0"
                    role="button"
                    class="flex items-center gap-2 cursor-pointer hover:text-primary transition-colors select-none"
                    title="Global Download Speed - Click to Set Limit"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 13.5L12 21m0 0l-7.5-7.5M12 21V3" />
                    </svg>
                    <span class="font-mono">{move || format_speed(stats.get().down_rate)}</span>
                    <Show when=move || { stats.get().down_limit.unwrap_or(0) > 0 } fallback=|| ()>
                        <span class="text-[10px] opacity-60">
                            {move || format!("(Limit: {})", format_speed(stats.get().down_limit.unwrap_or(0)))}
                        </span>
                    </Show>
                </div>

                <ul
                    tabindex="0"
                    class="dropdown-content z-[100] menu p-2 shadow bg-base-200 rounded-box w-40 mb-2 border border-base-300"
                >
                    {
                        limits.clone().into_iter().map(|(val, label)| {
                            let is_active = move || {
                                let current = stats.get().down_limit.unwrap_or(0);
                                (current - val).abs() < 1024
                            };
                            let close = close_dropdown.clone();
                            view! {
                                <li>
                                    <button
                                        class=move || if is_active() { "bg-primary/10 text-primary font-bold text-xs flex justify-between" } else { "text-xs flex justify-between" }
                                        on:click=move |_| {
                                            set_limit("down", val);
                                            close();
                                        }
                                    >
                                        {label}
                                        <Show when=is_active fallback=|| ()>
                                            <span>"✓"</span>
                                        </Show>
                                    </button>
                                </li>
                            }
                        }).collect::<Vec<_>>()
                    }
                </ul>
            </div>

            // --- UPLOAD SPEED DROPDOWN ---
            <div
                class="dropdown dropdown-top dropdown-start"
                 on:touchstart=move |e| e.stop_propagation()
            >
                <div
                    tabindex="0"
                    role="button"
                    class="flex items-center gap-2 cursor-pointer hover:text-primary transition-colors select-none"
                    title="Global Upload Speed - Click to Set Limit"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 10.5L12 3m0 0l7.5 7.5M12 3v18" />
                    </svg>
                    <span class="font-mono">{move || format_speed(stats.get().up_rate)}</span>
                    <Show when=move || { stats.get().up_limit.unwrap_or(0) > 0 } fallback=|| ()>
                        <span class="text-[10px] opacity-60">
                            {move || format!("(Limit: {})", format_speed(stats.get().up_limit.unwrap_or(0)))}
                        </span>
                    </Show>
                </div>

                <ul
                    tabindex="0"
                    class="dropdown-content z-[100] menu p-2 shadow bg-base-200 rounded-box w-40 mb-2 border border-base-300"
                >
                    {
                        limits.clone().into_iter().map(|(val, label)| {
                            let is_active = move || {
                                let current = stats.get().up_limit.unwrap_or(0);
                                (current - val).abs() < 1024
                            };
                            let close = close_dropdown.clone();
                            view! {
                                <li>
                                    <button
                                        class=move || if is_active() { "bg-primary/10 text-primary font-bold text-xs flex justify-between" } else { "text-xs flex justify-between" }
                                        on:click=move |_| {
                                            set_limit("up", val);
                                            close();
                                        }
                                    >
                                        {label}
                                        <Show when=is_active fallback=|| ()>
                                            <span>"✓"</span>
                                        </Show>
                                    </button>
                                </li>
                            }
                        }).collect::<Vec<_>>()
                    }
                </ul>
            </div>

            <div class="ml-auto flex items-center gap-4">
                <div
                    class="dropdown dropdown-top dropdown-end"
                     on:touchstart=move |e| e.stop_propagation()
                >
                    <div
                        tabindex="0"
                        role="button"
                        class="btn btn-ghost btn-xs btn-square"
                        title="Change Theme"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9.53 16.122a3 3 0 0 0-5.78 1.128 2.25 2.25 0 0 1-2.4 2.245 4.5 4.5 0 0 0 8.4-2.245c0-.399-.078-.78-.22-1.128Zm0 0a15.998 15.998 0 0 0 3.388-1.62m-5.043-.025a15.994 15.994 0 0 1 1.622-3.395m3.42 3.42a15.995 15.995 0 0 0 4.764-4.648l3.876-5.814a1.151 1.151 0 0 0-1.597-1.597L14.146 6.32a15.996 15.996 0 0 0-4.649 4.763m3.42 3.42a6.776 6.776 0 0 0-3.42-3.42" />
                        </svg>
                    </div>

                    <ul
                        tabindex="0"
                        class="dropdown-content z-[100] menu p-2 shadow bg-base-200 rounded-box w-52 mb-2 border border-base-300 max-h-96 overflow-y-auto"
                    >
                        {
                            let themes = vec![
                                "light", "dark", "dim", "nord", "cupcake", "dracula", "cyberpunk", "emerald", "sunset", "abyss"
                            ];
                            let close = close_dropdown.clone();
                            themes.into_iter().map(|theme| {
                                let close = close.clone();
                                view! {
                                    <li>
                                        <button
                                            class=move || if current_theme.get() == theme { "bg-primary/10 text-primary font-bold text-xs capitalize" } else { "text-xs capitalize" }
                                            on:click=move |_| {
                                                set_current_theme.set(theme.to_string());
                                                if let Some(win) = web_sys::window() {
                                                    if let Some(doc) = win.document() {
                                                        let _ = doc.document_element().unwrap().set_attribute("data-theme", theme);
                                                    }
                                                    if let Some(storage) = win.local_storage().ok().flatten() {
                                                        let _ = storage.set_item("vibetorrent_theme", theme);
                                                    }
                                                }
                                                close();
                                            }
                                        >
                                            {theme}
                                        </button>
                                    </li>
                                }
                            }).collect::<Vec<_>>()
                        }
                    </ul>
                </div>

                <button 
                    class="btn btn-ghost btn-xs btn-square" 
                    title="Settings & Notification Permissions"
                    on:click=move |_| {
                        // Request push notification permission when settings button is clicked
                        spawn_local(async {
                            log::info!("Settings button clicked - requesting push notification permission");
                            
                            // Check current permission state before requesting
                            let window = web_sys::window().expect("window should exist");
                            let _current_perm = js_sys::Reflect::get(&window, &"Notification".into())
                                .ok()
                                .and_then(|n| js_sys::Reflect::get(&n, &"permission".into()).ok())
                                .and_then(|p| p.as_string())
                                .unwrap_or_default();
                                
                            crate::store::subscribe_to_push_notifications().await;
                            
                            // Check permission after request
                            let new_perm = js_sys::Reflect::get(&window, &"Notification".into())
                                .ok()
                                .and_then(|n| js_sys::Reflect::get(&n, &"permission".into()).ok())
                                .and_then(|p| p.as_string())
                                .unwrap_or_default();
                            
                            if let Some(store) = use_context::<crate::store::TorrentStore>() {
                                if new_perm == "granted" {
                                    crate::store::show_toast_with_signal(
                                        store.notifications,
                                        shared::NotificationLevel::Success,
                                        "Bildirimler etkinleştirildi! Torrent tamamlandığında bildirim alacaksınız.".to_string(),
                                    );
                                } else if new_perm == "denied" {
                                    crate::store::show_toast_with_signal(
                                        store.notifications,
                                        shared::NotificationLevel::Error,
                                        "Bildirim izni reddedildi. Tarayıcı ayarlarından izin verebilirsiniz.".to_string(),
                                    );
                                } else {
                                    crate::store::show_toast_with_signal(
                                        store.notifications,
                                        shared::NotificationLevel::Warning,
                                        "Bildirim izni verilemedi. Açılan izin penceresinde 'İzin Ver' seçeneğini seçin.".to_string(),
                                    );
                                }
                            }
                        });
                    }
                >
                     <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.212 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                </button>
            </div>
        </div>
    }
}
