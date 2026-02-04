use leptos::*;
use shared::GlobalLimitRequest;

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
    let (theme_open, set_theme_open) = create_signal(false);

    // Dropdown states
    let (down_menu_open, set_down_menu_open) = create_signal(false);
    let (up_menu_open, set_up_menu_open) = create_signal(false);

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
        set_down_menu_open.set(false);
        set_up_menu_open.set(false);
    };

    // Register global click/touch listener to close menus when clicking outside
    let close_menus = move || {
        if down_menu_open.get_untracked() { set_down_menu_open.set(false); }
        if up_menu_open.get_untracked() { set_up_menu_open.set(false); }
        if theme_open.get_untracked() { set_theme_open.set(false); }
    };
    
    // Use window_event_listener from leptos for both click and touchstart (for iOS)
    let _ = window_event_listener(ev::click, {
        let close = close_menus.clone();
        move |_| close()
    });
    let _ = window_event_listener(ev::touchstart, move |_| close_menus());

    view! {
        <div class="h-8 min-h-8 bg-base-200 border-t border-base-300 flex items-center px-4 text-xs gap-4 text-base-content/70">

            // --- DOWNLOAD SPEED DROPDOWN ---
            <div class=move || {
                let base = "dropdown dropdown-top dropdown-start";
                if down_menu_open.get() { format!("{} dropdown-open", base) } else { base.to_string() }
            }>
                <div
                    tabindex="0"
                    role="button"
                    class="flex items-center gap-2 cursor-pointer hover:text-primary transition-colors select-none"
                    title="Global Download Speed - Click to Set Limit"
                    on:click=move |e| {
                        e.stop_propagation();
                        set_down_menu_open.update(|v| *v = !*v);
                        set_up_menu_open.set(false);
                        set_theme_open.set(false);
                    }
                    on:touchstart=move |e| {
                        e.stop_propagation();
                        // touchstart will trigger click usually, but we stop propagation here so window listener doesn't fire.
                        // We rely on click for the toggle logic to avoid double toggle.
                    }
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
                    on:click=move |e| e.stop_propagation()
                    on:touchstart=move |e| e.stop_propagation()
                >
                    {
                        limits.clone().into_iter().map(|(val, label)| {
                            let is_active = move || {
                                let current = stats.get().down_limit.unwrap_or(0);
                                (current - val).abs() < 1024
                            };
                            view! {
                                <li>
                                    <button
                                        class=move || if is_active() { "bg-primary/10 text-primary font-bold text-xs flex justify-between" } else { "text-xs flex justify-between" }
                                        on:click=move |_| {
                                            set_limit("down", val);
                                            set_down_menu_open.set(false);
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
            <div class=move || {
                let base = "dropdown dropdown-top dropdown-start";
                if up_menu_open.get() { format!("{} dropdown-open", base) } else { base.to_string() }
            }>
                <div
                    tabindex="0"
                    role="button"
                    class="flex items-center gap-2 cursor-pointer hover:text-primary transition-colors select-none"
                    title="Global Upload Speed - Click to Set Limit"
                    on:click=move |e| {
                        e.stop_propagation();
                        set_up_menu_open.update(|v| *v = !*v);
                        set_down_menu_open.set(false);
                        set_theme_open.set(false);
                    }
                    on:touchstart=move |e| {
                        e.stop_propagation();
                    }
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 11.25l-3-3m0 0l-3 3m3-3v7.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
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
                    on:click=move |e| e.stop_propagation()
                    on:touchstart=move |e| e.stop_propagation()
                >
                    {
                        limits.clone().into_iter().map(|(val, label)| {
                            let is_active = move || {
                                let current = stats.get().up_limit.unwrap_or(0);
                                (current - val).abs() < 1024
                            };
                            view! {
                                <li>
                                    <button
                                        class=move || if is_active() { "bg-primary/10 text-primary font-bold text-xs flex justify-between" } else { "text-xs flex justify-between" }
                                        on:click=move |_| {
                                            set_limit("up", val);
                                            set_up_menu_open.set(false);
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
                <div class=move || {
                    let base = "dropdown dropdown-top dropdown-end";
                    if theme_open.get() {
                        format!("{} dropdown-open", base)
                    } else {
                        base.to_string()
                    }
                }>
                    <div
                        tabindex="0"
                        role="button"
                        class="btn btn-ghost btn-xs btn-square"
                        title="Change Theme"
                        on:click=move |e| {
                            e.stop_propagation();
                            set_theme_open.update(|v| *v = !*v);
                            set_down_menu_open.set(false);
                            set_up_menu_open.set(false);
                        }
                        on:touchstart=move |e| {
                            e.stop_propagation();
                        }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M4.098 19.902a3.75 3.75 0 0 0 5.304 0l6.401-6.402M6.75 21A3.75 3.75 0 0 1 3 17.25V4.125C3 3.504 3.504 3 4.125 3h5.25c.621 0 1.125.504 1.125 1.125v4.072c0 .657.66 1.175 1.312 1.133 3.421-.22 6.187 2.546 6.187 5.965 0 1.595-.572 3.064-1.524 4.195" />
                        </svg>
                    </div>

                    <ul 
                        tabindex="0" 
                        class="dropdown-content z-[100] menu p-2 shadow bg-base-200 rounded-box w-52 mb-2 border border-base-300 max-h-96 overflow-y-auto block"
                        on:click=move |e| e.stop_propagation()
                        on:touchstart=move |e| e.stop_propagation()
                    >
                        {
                            let themes = vec![
                                "light", "dark", "cupcake", "bumblebee", "emerald", "corporate", "synthwave", "retro", "cyberpunk", "valentine", "halloween", "garden", "forest", "aqua", "lofi", "pastel", "fantasy", "wireframe", "black", "luxury", "dracula", "cmyk", "autumn", "business", "acid", "lemonade", "night", "coffee", "winter", "dim", "nord", "sunset"
                            ];
                            themes.into_iter().map(|theme| {
                                view! {
                                    <li>
                                        <button
                                            class=move || if theme == "dim" { "bg-primary/10 text-primary font-bold text-xs capitalize" } else { "text-xs capitalize" }
                                            on:click=move |_| {
                                                let doc = web_sys::window().unwrap().document().unwrap();
                                                let _ = doc.document_element().unwrap().set_attribute("data-theme", theme);
                                                set_theme_open.set(false);
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

                <button class="btn btn-ghost btn-xs btn-square" title="Settings">
                     <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.212 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                </button>
            </div>
        </div>
    }
}
