use leptos::prelude::*;
use leptos::html;
use leptos_use::storage::use_local_storage;
use ::codee::string::FromToStringCodec;
use shared::GlobalLimitRequest;
use crate::api;

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

    let (current_theme, set_current_theme, _) = use_local_storage::<String, FromToStringCodec>("vibetorrent_theme");

    // Initialize with default if empty
    let current_theme_val = current_theme.get();
    if current_theme_val.is_empty() {
        set_current_theme.set("dark".to_string());
    }

    // Automatically sync theme to document attribute
    Effect::new(move |_| {
        let theme = current_theme.get().to_lowercase();
        if let Some(doc) = document().document_element() {
            let _ = doc.set_attribute("data-theme", &theme);
            // Also set class for Shadcn dark mode support
            if theme == "dark" || theme == "dracula" || theme == "dim" || theme == "abyss" {
                let _ = doc.class_list().add_1("dark");
            } else {
                let _ = doc.class_list().remove_1("dark");
            }
        }
    });

    // Preset limits in bytes/s
    let limits: Vec<(i64, &str)> = vec!(
        (0, "Unlimited"),
        (100 * 1024, "100 KB/s"),
        (500 * 1024, "500 KB/s"),
        (1024 * 1024, "1 MB/s"),
        (2 * 1024 * 1024, "2 MB/s"),
        (5 * 1024 * 1024, "5 MB/s"),
        (10 * 1024 * 1024, "10 MB/s"),
        (20 * 1024 * 1024, "20 MB/s"),
    );

    let set_limit = move |limit_type: &str, val: i64| {
        let limit_type = limit_type.to_string();
        log::info!("Setting {} limit to {}", limit_type, val);

        let req = if limit_type == "down" {
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

        leptos::task::spawn_local(async move {
            if let Err(e) = api::settings::set_global_limits(&req).await {
                log::error!("Failed to set limit: {:?}", e);
            } else {
                log::info!("Limit set successfully");
            }
        });
    };

        let down_details_ref = NodeRef::<html::Details>::new();
        let up_details_ref = NodeRef::<html::Details>::new();
        let theme_details_ref = NodeRef::<html::Details>::new();

        let close_details = move |node_ref: NodeRef<html::Details>| {
            if let Some(el) = node_ref.get_untracked() {
                el.set_open(false);
            }
        };

        view! {
            <div class="fixed bottom-0 left-0 right-0 h-8 min-h-8 bg-muted border-t border-border flex items-center px-4 text-xs gap-4 text-muted-foreground z-[99] cursor-pointer">

                // --- DOWNLOAD SPEED DROPDOWN ---
                <details class="group relative" node_ref=down_details_ref>
                    <summary class="flex items-center gap-2 cursor-pointer hover:text-foreground transition-colors select-none list-none [&::-webkit-details-marker]:hidden outline-none">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 13.5L12 21m0 0l-7.5-7.5M12 21V3" />
                        </svg>
                        <span class="font-mono">{move || format_speed(stats.get().down_rate)}</span>
                        <Show when=move || { stats.get().down_limit.unwrap_or(0) > 0 } fallback=|| ()>
                            <span class="text-[10px] opacity-60">
                                {move || format!("(Limit: {})", format_speed(stats.get().down_limit.unwrap_or(0)))}
                            </span>
                        </Show>
                    </summary>

                    <div class="absolute bottom-full left-0 mb-2 z-[100] min-w-[8rem] overflow-hidden rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md hidden group-open:block animate-in fade-in-0 zoom-in-95 slide-in-from-bottom-2">
                        <ul class="w-full">
                            {
                                limits.clone().into_iter().map(|(val, label)| {
                                    let is_active = move || {
                                        let current = stats.get().down_limit.unwrap_or(0);
                                        (current - val).abs() < 1024
                                    };
                                    view! {
                                        <li>
                                            <button
                                                class=move || {
                                                    let base = "relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-xs outline-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 hover:bg-accent hover:text-accent-foreground";
                                                    if is_active() { format!("{} bg-accent text-accent-foreground font-medium", base) } else { base.to_string() }
                                                }
                                                on:click=move |_| {
                                                    set_limit("down", val);
                                                    close_details(down_details_ref);
                                                }
                                            >
                                                <span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
                                                    <Show when=is_active fallback=|| ()>
                                                        <span>"✓"</span>
                                                    </Show>
                                                </span>
                                                {label}
                                            </button>
                                        </li>
                                    }
                                }).collect::<Vec<_>>()
                            }
                        </ul>
                    </div>
                </details>

                // --- UPLOAD SPEED DROPDOWN ---
                <details class="group relative" node_ref=up_details_ref>
                    <summary class="flex items-center gap-2 cursor-pointer hover:text-foreground transition-colors select-none list-none [&::-webkit-details-marker]:hidden outline-none">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 10.5L12 3m0 0l7.5 7.5M12 3v18" />
                        </svg>
                        <span class="font-mono">{move || format_speed(stats.get().up_rate)}</span>
                        <Show when=move || { stats.get().up_limit.unwrap_or(0) > 0 } fallback=|| ()>
                            <span class="text-[10px] opacity-60">
                                {move || format!("(Limit: {})", format_speed(stats.get().up_limit.unwrap_or(0)))}
                            </span>
                        </Show>
                    </summary>

                    <div class="absolute bottom-full left-0 mb-2 z-[100] min-w-[8rem] overflow-hidden rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md hidden group-open:block animate-in fade-in-0 zoom-in-95 slide-in-from-bottom-2">
                        <ul class="w-full">
                            {
                                limits.clone().into_iter().map(|(val, label)| {
                                    let is_active = move || {
                                        let current = stats.get().up_limit.unwrap_or(0);
                                        (current - val).abs() < 1024
                                    };
                                    view! {
                                        <li>
                                            <button
                                                class=move || {
                                                    let base = "relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-xs outline-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 hover:bg-accent hover:text-accent-foreground";
                                                    if is_active() { format!("{} bg-accent text-accent-foreground font-medium", base) } else { base.to_string() }
                                                }
                                                on:click=move |_| {
                                                    set_limit("up", val);
                                                    close_details(up_details_ref);
                                                }
                                            >
                                                <span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
                                                    <Show when=is_active fallback=|| ()>
                                                        <span>"✓"</span>
                                                    </Show>
                                                </span>
                                                {label}
                                            </button>
                                        </li>
                                    }
                                }).collect::<Vec<_>>()
                            }
                        </ul>
                    </div>
                </details>

                <div class="ml-auto flex items-center gap-4">
                    <details class="group relative" node_ref=theme_details_ref>
                        <summary class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-7 w-7 cursor-pointer outline-none list-none [&::-webkit-details-marker]:hidden">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M9.53 16.122a3 3 0 0 0-5.78 1.128 2.25 2.25 0 0 1-2.4 2.245 4.5 4.5 0 0 0 8.4-2.245c0-.399-.078-.78-.22-1.128Zm0 0a15.998 15.998 0 0 0 3.388-1.62m-5.043-.025a15.994 15.994 0 0 1 1.622-3.395m3.42 3.42a15.995 15.995 0 0 0 4.764-4.648l3.876-5.814a1.151 1.151 0 0 0-1.597-1.597L14.146 6.32a15.996 15.996 0 0 0-4.649 4.763m3.42 3.42a6.776 6.776 0 0 0-3.42-3.42" />
                            </svg>
                        </summary>

                        <div class="absolute bottom-full right-0 mb-2 z-[100] min-w-[8rem] overflow-hidden rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md hidden group-open:block animate-in fade-in-0 zoom-in-95 slide-in-from-bottom-2 max-h-96 overflow-y-auto">
                            <ul class="w-full">
                                {
                                    let themes = vec![
                                        "light", "dark", "dim", "nord", "cupcake", "dracula", "cyberpunk", "emerald", "sunset", "abyss"
                                    ];
                                    themes.into_iter().map(|theme| {
                                        let theme_name = theme.to_string();
                                        let theme_name_for_class = theme_name.clone();
                                        let theme_name_for_onclick = theme_name.clone();
                                        let is_active = move || current_theme.get() == theme_name_for_class;
                                        view! {
                                            <li>
                                                <button
                                                    class=move || {
                                                        let base = "relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-xs outline-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 hover:bg-accent hover:text-accent-foreground capitalize";
                                                        if is_active() { format!("{} bg-accent text-accent-foreground font-medium", base) } else { base.to_string() }
                                                    }
                                                    on:click=move |_| {
                                                        set_current_theme.set(theme_name_for_onclick.clone());
                                                        close_details(theme_details_ref);
                                                    }
                                                >
                                                    <span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
                                                        <Show when=is_active fallback=|| ()>
                                                            <span>"✓"</span>
                                                        </Show>
                                                    </span>
                                                    {theme_name}
                                                </button>
                                            </li>
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            </ul>
                        </div>
                    </details>
                <button
                    class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-7 w-7"
                    title="Settings & Notification Permissions"
                    on:click=move |_| {
                        // Request push notification permission
                        leptos::task::spawn_local(async {
                            // ... existing logic ...
                            crate::store::subscribe_to_push_notifications().await;
                            // ... existing logic ...
                        });
                    }
                >
                     <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.212 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 012.6-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                </button>
            </div>
        </div>
    }
}