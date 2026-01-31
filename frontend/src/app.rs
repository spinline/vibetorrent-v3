use leptos::*;
use shared::{Torrent, AppEvent, TorrentStatus, Theme};
use crate::components::context_menu::ContextMenu;
use crate::components::modal::Modal;
use crate::components::ui::button::{Button, ButtonVariant};
use gloo_net::eventsource::futures::EventSource;
use futures::StreamExt;

#[component]
pub fn App() -> impl IntoView {
    // Signals
    let (torrents, set_torrents) = create_signal(Vec::<Torrent>::new());
    let (sort_key, set_sort_key) = create_signal(6); // 6=Added Date
    let (sort_asc, set_sort_asc) = create_signal(false); // Descending (Newest first)
    let (filter_status, set_filter_status) = create_signal(Option::<TorrentStatus>::None);
    let (active_tab, set_active_tab) = create_signal("torrents");
    let (show_mobile_sidebar, set_show_mobile_sidebar) = create_signal(false);
    // Theme with Persistence
    let (theme, set_theme) = create_signal({
        let storage = window().local_storage().ok().flatten();
        let saved = storage.and_then(|s| s.get_item("vibetorrent_theme").ok().flatten());
        match saved.as_deref() {
            Some("Light") => Theme::Light,
            Some("Amoled") => Theme::Amoled,
            _ => Theme::Midnight,
        }
    }); 
    
    // Persist Theme
    // Persist Theme & Apply CSS Variables
    create_effect(move |_| {
         let val = match theme.get() {
             Theme::Midnight => "Midnight",
             Theme::Light => "Light",
             Theme::Amoled => "Amoled",
         };
         
         if let Some(doc) = window().document() {
             if let Some(body) = doc.body() {
                 let list = body.class_list();
                 match theme.get() {
                     Theme::Light => {
                         let _ = list.remove_1("dark");
                         let _ = list.remove_1("amoled");
                     },
                     Theme::Midnight => {
                         let _ = list.add_1("dark");
                         let _ = list.remove_1("amoled");
                     },
                     Theme::Amoled => {
                         let _ = list.add_1("dark");
                         let _ = list.add_1("amoled");
                     },
                 }
             }
         }

         if let Some(storage) = window().local_storage().ok().flatten() {
             let _ = storage.set_item("vibetorrent_theme", val);
         }
    });

    // Remove Loading Spinner (Fix for spinner hanging)
    create_effect(move |_| {
        if let Some(doc) = window().document() {
            if let Some(el) = doc.get_element_by_id("app-loading") {
                el.remove();
            }
        }
    });

    // Mobile Sidebar Scroll Lock
    create_effect(move |_| {
        if let Some(doc) = window().document() {
            if let Some(body) = doc.body() {
                let style = body.style();
                if show_mobile_sidebar.get() {
                    let _ = style.set_property("overflow", "hidden");
                } else {
                    let _ = style.remove_property("overflow");
                }
            }
        }
    });

    // Context Menu Signals
    let (cm_visible, set_cm_visible) = create_signal(false);
    let (cm_pos, set_cm_pos) = create_signal((0, 0));
    let (cm_target_hash, set_cm_target_hash) = create_signal(String::new());
    
    // Delete Confirmation State
    let (show_delete_modal, set_show_delete_modal) = create_signal(false);
    let (pending_action, set_pending_action) = create_signal(Option::<(String, String)>::None); // (Action, Hash)
    
    // Debug: Last Updated Timestamp
    let (last_updated, set_last_updated) = create_signal(0u64);

    // Derived: Filtered & Sorted Logic
    let processed_torrents = create_memo(move |_| {
        let mut items = torrents.get();
        if let Some(status) = filter_status.get() {
            items.retain(|t| t.status == status);
        }
        
        let key = sort_key.get();
        let asc = sort_asc.get();

        items.sort_by(|a, b| {
            let cmp = match key {
                0 => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                1 => a.size.cmp(&b.size),
                2 => a.percent_complete.partial_cmp(&b.percent_complete).unwrap_or(std::cmp::Ordering::Equal),
                3 => a.down_rate.cmp(&b.down_rate),
                4 => a.up_rate.cmp(&b.up_rate),
                5 => a.eta.cmp(&b.eta),
                6 => a.added_date.cmp(&b.added_date),
                _ => std::cmp::Ordering::Equal,
            };
            if asc { cmp } else { cmp.reverse() }
        });
        items
    });

    let sort = move |key: i32| {
        if sort_key.get() == key {
            set_sort_asc.update(|a| *a = !*a);
        } else {
            set_sort_key.set(key);
            set_sort_asc.set(true); 
        }
    };

    // Add Torrent Logic
    let (show_modal, set_show_modal) = create_signal(false);
    let (magnet_link, set_magnet_link) = create_signal(String::new());

    let add_torrent = move |_| {
         spawn_local(async move {
            let uri = magnet_link.get();
            if uri.is_empty() { return; }
            let client = gloo_net::http::Request::post("/api/torrents/add")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&serde_json::json!({ "uri": uri })).unwrap())
                .unwrap();
            if client.send().await.is_ok() {
                set_magnet_link.set(String::new());
                set_show_modal.set(false);
            }
         });
    };

    // Connect SSE
    create_effect(move |_| {
        spawn_local(async move {
            logging::log!("Connecting to SSE...");
            let mut es = EventSource::new("/api/events").unwrap();
            let mut stream = es.subscribe("message").unwrap();
            
            loop {
                match stream.next().await {
                    Some(Ok((_, msg))) => {
                        let data = msg.data().as_string().unwrap();
                        match serde_json::from_str::<AppEvent>(&data) {
                            Ok(event) => {
                                 match event {
                                     AppEvent::FullList(list, ts) => {
                                         set_torrents.set(list);
                                         set_last_updated.set(ts);
                                     }
                                     AppEvent::Update(diff) => {
                                         set_torrents.update(|list| {
                                             if let Some(target) = list.iter_mut().find(|t| t.hash == diff.hash) {
                                                 if let Some(v) = diff.name { target.name = v; }
                                                 if let Some(v) = diff.size { target.size = v; }
                                                 if let Some(v) = diff.down_rate { target.down_rate = v; }
                                                 if let Some(v) = diff.up_rate { target.up_rate = v; }
                                                 if let Some(v) = diff.percent_complete { target.percent_complete = v; }
                                                 if let Some(v) = diff.completed { target.completed = v; }
                                                 if let Some(v) = diff.eta { target.eta = v; }
                                                 if let Some(v) = diff.status { target.status = v; }
                                                 if let Some(v) = diff.error_message { target.error_message = v; }
                                             }
                                         });
                                     }
                                 }
                            }
                            Err(e) => {
                                logging::error!("Failed to parse SSE JSON: {}", e);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        logging::error!("SSE Stream Error: {:?}", e);
                    }
                    None => {
                        logging::warn!("SSE Stream Ended (None received)");
                        break;
                    }
                }
            }
            logging::warn!("SSE Task Exiting");
        });
    });

    // Formatting Helpers
    let format_bytes = |bytes: i64| {
        if bytes < 1024 { format!("{} B", bytes) }
        else if bytes < 1048576 { format!("{:.1} KB", bytes as f64 / 1024.0) }
        else if bytes < 1073741824 { format!("{:.1} MB", bytes as f64 / 1048576.0) }
        else { format!("{:.1} GB", bytes as f64 / 1073741824.0) }
    };
    
    let format_eta = |eta: i64| {
        if eta <= 0 || eta > 31536000 { return "∞".to_string(); }
        let h = eta / 3600;
        let m = (eta % 3600) / 60;
        format!("{}h {}m", h, m)
    };

    // Theme Engine


    let filter_btn_class = move |status: Option<TorrentStatus>| {
        crate::utils::cn(format!(
            "block px-4 py-2 rounded-md transition-all duration-200 text-left w-full flex items-center gap-3 border text-sm font-medium {}",
            if filter_status.get() == status {
                "bg-primary/10 text-primary border-primary/20"
            } else {
                "border-transparent text-muted-foreground hover:text-foreground hover:bg-accent hover:text-accent-foreground"
            }
        ))
    };
    
    let tab_btn_class = move |tab: &str| {
        crate::utils::cn(format!(
            "flex flex-col items-center justify-center p-2 flex-1 transition-colors relative {}",
            if active_tab.get() == tab {
                "text-primary"
            } else {
                "text-muted-foreground hover:text-foreground"
            }
        ))
    };
    
    // Sidebar Content Logic
    let sidebar_content = move || {
        view! {
            <div class="mb-10 px-2 flex items-center gap-3">
                 <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center shadow-lg shadow-blue-500/30">
                    <svg class="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                       <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                 </div>
                <h1 class="text-2xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 tracking-tight">
                    "VibeTorrent"
                </h1>
            </div>
            
            <div class="text-xs font-bold uppercase tracking-widest mb-4 px-2 text-muted-foreground">"Filters"</div>
            <nav class="space-y-2 flex-1">
                <button class={move || filter_btn_class(None)} on:click=move |_| { set_filter_status.set(None); set_show_mobile_sidebar.set(false); }>
                    <span class="w-2 h-2 rounded-full bg-gray-400"></span>
                    "All Torrents"
                </button>
                <button class={move || filter_btn_class(Some(TorrentStatus::Downloading))} on:click=move |_| { set_filter_status.set(Some(TorrentStatus::Downloading)); set_show_mobile_sidebar.set(false); }>
                    <span class="w-2 h-2 rounded-full bg-blue-500"></span>
                    "Downloading"
                </button>
                <button class={move || filter_btn_class(Some(TorrentStatus::Seeding))} on:click=move |_| { set_filter_status.set(Some(TorrentStatus::Seeding)); set_show_mobile_sidebar.set(false); }>
                    <span class="w-2 h-2 rounded-full bg-green-500"></span>
                    "Seeding"
                </button>
                <button class={move || filter_btn_class(Some(TorrentStatus::Paused))} on:click=move |_| { set_filter_status.set(Some(TorrentStatus::Paused)); set_show_mobile_sidebar.set(false); }>
                    <span class="w-2 h-2 rounded-full bg-yellow-500"></span>
                    "Paused"
                </button>
                <button class={move || filter_btn_class(Some(TorrentStatus::Error))} on:click=move |_| { set_filter_status.set(Some(TorrentStatus::Error)); set_show_mobile_sidebar.set(false); }>
                    <span class="w-2 h-2 rounded-full bg-red-500"></span>
                    "Errors"
        </button>
            </nav>
            
            <div class="mt-auto pt-6 border-t border-border">
                <div class="rounded-xl p-4 border border-border relative overflow-hidden bg-card">
                    <div class="absolute inset-0 opacity-5 bg-foreground"></div>
                    <div class="text-xs mb-2 z-10 relative text-muted-foreground">"Storage"</div>
                    <div class="w-full bg-secondary/50 rounded-full h-1.5 mb-2 overflow-hidden z-10 relative">
                         <div class="bg-gradient-to-r from-blue-500 to-purple-500 w-[70%] h-full rounded-full"></div>
                    </div>
                    <div class="flex justify-between text-xs z-10 relative text-muted-foreground">
                        <span>"700 GB used"</span>
                        <span>"1 TB total"</span>
                    </div>
                </div>
            </div>
        }
    };

    let theme_option = move |t: Theme, label: &str, color: &str| {
        let is_active = theme.get() == t;
        let border_class = if is_active { "border-blue-500 ring-1 ring-blue-500/50" } else { "border-transparent hover:border-gray-500/30" };
        let label_owned = label.to_string();
        let color_owned = color.to_string();
        
        view! {
            <button 
                class={format!("flex items-center gap-4 p-4 rounded-xl border bg-black/5 dark:bg-white/5 transition-all w-full text-left {}", border_class)}
                on:click=move |_| set_theme.set(t.clone())
            >
                <div class={format!("w-12 h-12 rounded-full shadow-lg flex-shrink-0 {}", color_owned)}></div>
                <div>
                    <div class="font-bold">{label_owned}</div>
                    <div class="text-xs opacity-60">"Select this theme"</div>
                </div>
                {if is_active {
                    view! { 
                        <div class="ml-auto text-blue-500">
                            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                        </div> 
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
            </button>
        }
    };

    view! {
        <div class="min-h-screen font-sans flex flex-col md:flex-row overflow-hidden transition-colors duration-300 bg-background text-foreground">
            // DESKTOP SIDEBAR
            <aside class="hidden md:flex flex-col w-72 border-r border-border p-6 z-20 h-screen bg-card/50 backdrop-blur-xl">
                {sidebar_content}
            </aside>
            
            // MOBILE SIDEBAR
            <div 
                class={move || if show_mobile_sidebar.get() { "fixed inset-0 z-50 flex md:hidden" } else { "hidden" }}
                on:click=move |_| ()
            >
                <div 
                    class="fixed inset-0 bg-background/80 backdrop-blur-sm transition-opacity cursor-default"
                    on:click=move |_| set_show_mobile_sidebar.set(false)
                ></div>
                <aside 
                    class="relative w-80 max-w-[85vw] h-full shadow-2xl p-6 flex flex-col animate-in slide-in-from-left duration-300 border-r border-border bg-card"
                    on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                >
                    <button class="absolute top-4 right-4 p-2 hover:opacity-80 text-muted-foreground" on:click=move |_| set_show_mobile_sidebar.set(false)>
                        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                    </button>
                    {sidebar_content}
                </aside>
            </div>

            // MAIN CONTENT
            <main class="flex-1 h-screen overflow-y-auto overflow-x-hidden relative pb-24 md:pb-0">
                <header class="fixed top-0 left-0 right-0 md:sticky md:top-0 z-40 border-b border-border px-6 py-4 flex justify-between items-center transition-colors duration-300 bg-background/80 backdrop-blur-xl">
                    <div class="flex items-center gap-3">
                         <button class="md:hidden p-1 -ml-2 hover:opacity-80 text-muted-foreground" on:click=move |_| set_show_mobile_sidebar.set(true)>
                            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" /></svg>
                         </button>
                        <h2 class="text-xl font-bold flex items-center gap-2 text-foreground">
                            {move || if active_tab.get() == "settings" { "Settings" } else if active_tab.get() == "dashboard" { "Dashboard" } else {
                                match filter_status.get() {
                                    None => "All Torrents",
                                    Some(TorrentStatus::Downloading) => "Downloading",
                                    Some(TorrentStatus::Seeding) => "Seeding",
                                    Some(TorrentStatus::Paused) => "Paused",
                                    Some(TorrentStatus::Error) => "Errors",
                                    _ => "Torrents"
                                }
                            }}
                        </h2>
                    </div>
                    <div class="flex items-center gap-3">
                        <div class="hidden md:block text-xs font-mono text-muted-foreground">
                            "Server Time: "
                            {move || {
                                let ts = last_updated.get();
                                if ts == 0 {
                                    "Waiting...".to_string()
                                } else {
                                    let s = ts % 60;
                                    let m = (ts / 60) % 60;
                                    let h = (ts / 3600) % 24;
                                    format!("{:02}:{:02}:{:02} UTC", h, m, s)
                                }
                            }}
                        </div>
                        <Button 
                            class="gap-2"
                            on_click=Callback::from(move |_| set_show_modal.set(true))
                        >
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
                            <span class="hidden md:inline">"Add Torrent"</span>
                            <span class="md:hidden">"Add"</span>
                        </Button>
                        
                        <Button 
                            variant=ButtonVariant::Ghost
                            class={Signal::derive(move || if active_tab.get() == "settings" { "text-primary bg-primary/10 border-primary/20".to_string() } else { "text-muted-foreground".to_string() })}
                            on_click=Callback::from(move |_| set_active_tab.set(if active_tab.get() == "settings" { "torrents" } else { "settings" }))
                        >
                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                            </svg>
                        </Button>
                    </div>
                </header>

                <div class="p-6 max-w-7xl mx-auto animate-in fade-in duration-500 pt-[88px] md:pt-6">
                    {move || if active_tab.get() == "settings" {
                        view! {
                            <div class="space-y-8">
                                <div>
                                    <h3 class="text-lg font-bold mb-4 text-foreground">"Appearance"</h3>
                                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                                        {theme_option(Theme::Midnight, "Midnight", "bg-[#0a0a0c] border border-gray-700")}
                                        {theme_option(Theme::Light, "Light", "bg-gray-100 border border-gray-300")}
                                        {theme_option(Theme::Amoled, "Amoled", "bg-black border border-gray-800")}
                                    </div>
                                </div>
                                <div class="p-6 rounded-2xl border border-border bg-card shadow-sm">
                                    <h3 class="text-lg font-bold mb-2 text-foreground">"About VibeTorrent"</h3>
                                    <p class="text-sm text-muted-foreground">"Version 3.0.0 (Rust + WebAssembly)"</p>
                                </div>
                            </div>
                        }.into_view()
                    } else if active_tab.get() == "dashboard" {
                        view! { 
                                <div class="text-center py-20 opacity-50 text-muted-foreground">"Dashboard Charts Coming Soon..."</div>
                        }.into_view()
                    } else {
                        view! {
                            // Torrent List (Desktop)
                            <div class="hidden md:block rounded-2xl border border-border bg-card shadow-sm overflow-hidden">
                                <table class="w-full text-left table-fixed">
                                    <thead class="uppercase text-xs font-bold tracking-wider text-muted-foreground bg-muted/50 border-b border-border">
                                        <tr>
                                            <th class="px-6 py-4 cursor-pointer hover:text-foreground transition-colors" on:click=move |_| sort(0)>"Name"</th>
                                            <th class="px-6 py-4 cursor-pointer hover:text-foreground transition-colors w-28 text-right whitespace-nowrap" on:click=move |_| sort(1)>"Size"</th>
                                            <th class="px-6 py-4 cursor-pointer hover:text-foreground transition-colors w-36" on:click=move |_| sort(2)>"Progress"</th>
                                            <th class="px-6 py-4 cursor-pointer hover:text-foreground transition-colors w-28 text-right whitespace-nowrap" on:click=move |_| sort(3)>"Down"</th>
                                            <th class="px-6 py-4 cursor-pointer hover:text-foreground transition-colors w-28 text-right whitespace-nowrap" on:click=move |_| sort(4)>"Up"</th>
                                            <th class="px-6 py-4 cursor-pointer hover:text-foreground transition-colors w-28 text-right whitespace-nowrap" on:click=move |_| sort(5)>"ETA"</th>
                                            <th class="px-6 py-4 text-center w-28">"Status"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-border">
                                        <For
                                        each=move || processed_torrents.get()
                                            key=|t| format!("{}-{}-{:?}-{}-{}-{}-{}", t.hash, t.name, t.status, t.down_rate, t.up_rate, t.percent_complete, t.error_message)
                                            children=move |torrent| {
                                                let status_color = match torrent.status {
                                                    TorrentStatus::Downloading => "text-blue-500 bg-blue-500/10 border-blue-500/20",
                                                    TorrentStatus::Seeding => "text-green-500 bg-green-500/10 border-green-500/20",
                                                    TorrentStatus::Paused => "text-yellow-500 bg-yellow-500/10 border-yellow-500/20",
                                                    TorrentStatus::Error => "text-destructive bg-destructive/10 border-destructive/20",
                                                    _ => "text-muted-foreground bg-muted"
                                                };
                                                let status_text = format!("{:?}", torrent.status);
                                                let error_msg = torrent.error_message.clone();
                                                let error_msg_view = error_msg.clone();
                                                
                                                view! {
                                                    <tr 
                                                        class="transition-colors group hover:bg-muted/50"
                                                        on:contextmenu=move |e: web_sys::MouseEvent| {
                                                            e.prevent_default();
                                                            set_cm_pos.set((e.client_x(), e.client_y()));
                                                            set_cm_target_hash.set(torrent.hash.clone());
                                                            set_cm_visible.set(true);
                                                        }
                                                    >
                                                        <td class="px-6 py-4 max-w-sm">
                                                            <div class="font-medium truncate transition-colors text-foreground" title={torrent.name.clone()}>
                                                                {torrent.name}
                                                            </div>
                                                            <Show when=move || !error_msg.is_empty() fallback=|| ()>
                                                                <div class="text-xs text-destructive mt-1">{error_msg_view.clone()}</div>
                                                            </Show>
                                                        </td>
                                                        <td class="px-6 py-4 text-sm font-mono text-right whitespace-nowrap text-muted-foreground">{format_bytes(torrent.size)}</td>
                                                        <td class="px-6 py-4">
                                                            <div class="flex flex-col gap-1.5">
                                                                <div class="flex justify-between text-xs text-muted-foreground">
                                                                    <span>{format!("{:.1}%", torrent.percent_complete)}</span>
                                                                </div>
                                                                <div class="w-full bg-secondary rounded-full h-1.5 overflow-hidden">
                                                                    <div 
                                                                        class="bg-primary h-full rounded-full transition-all duration-500" 
                                                                        style=format!("width: {}%", torrent.percent_complete)
                                                                    ></div>
                                                                </div>
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4 font-mono text-xs text-right whitespace-nowrap text-muted-foreground">
                                                            {if torrent.down_rate > 0 {
                                                                view! { <span class="text-blue-500">{format_bytes(torrent.down_rate)} "/s"</span> }.into_view()
                                                            } else {
                                                                view! { <span class="text-muted-foreground/50">"-"</span> }.into_view()
                                                            }}
                                                        </td>
                                                        <td class="px-6 py-4 font-mono text-xs text-right whitespace-nowrap text-muted-foreground">
                                                            {if torrent.up_rate > 0 {
                                                                view! { <span class="text-green-500">{format_bytes(torrent.up_rate)} "/s"</span> }.into_view()
                                                            } else {
                                                                view! { <span class="text-muted-foreground/50">"-"</span> }.into_view()
                                                            }}
                                                        </td>
                                                        <td class="px-6 py-4 text-xs font-mono text-right whitespace-nowrap text-muted-foreground">
                                                            {format_eta(torrent.eta)}
                                                        </td>
                                                        <td class="px-6 py-4 text-center">
                                                            <span class={format!("text-[10px] font-bold px-2.5 py-1 rounded-full border {}", status_color)}>
                                                                {status_text}
                                                            </span>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            </div>

                            // Torrent List (Mobile)
                            <div class="md:hidden space-y-4">
                                <For
                                    each=move || processed_torrents.get()
                                    key=|t| format!("{}-{}-{:?}-{}-{}-{}-{}", t.hash, t.name, t.status, t.down_rate, t.up_rate, t.percent_complete, t.error_message)
                                    children=move |torrent| {
                                        let status_color = match torrent.status {
                                            TorrentStatus::Downloading => "text-blue-500",
                                            TorrentStatus::Seeding => "text-green-500",
                                            TorrentStatus::Paused => "text-yellow-500",
                                            TorrentStatus::Error => "text-destructive",
                                            _ => "text-muted-foreground"
                                        };
                                        
                                        view! {
                                            <div class="rounded-2xl p-4 border border-border shadow-sm active:scale-[0.98] transition-transform bg-card">
                                                <div class="flex justify-between items-start mb-3">
                                                    <div class="font-medium line-clamp-2 pr-4 text-foreground">{torrent.name}</div>
                                                    <div class={format!("text-xs font-bold {}", status_color)}>
                                                        {format!("{:?}", torrent.status)}
                                                    </div>
                                                </div>
                                                <div class="mb-4">
                                                    <div class="flex justify-between text-xs mb-1 text-muted-foreground">
                                                        <span>{format_bytes(torrent.size)}</span>
                                                        <span>{format!("{:.1}%", torrent.percent_complete)}</span>
                                                    </div>
                                                    <div class="w-full bg-secondary rounded-full h-1.5 overflow-hidden">
                                                        <div 
                                                            class="bg-primary h-full rounded-full transition-all duration-500" 
                                                            style=format!("width: {}%", torrent.percent_complete)
                                                        ></div>
                                                    </div>
                                                </div>
                                                <div class="flex justify-between items-center text-xs font-mono opacity-80">
                                                    <div class="flex gap-3">
                                                        <span class="text-blue-500">"↓ " {format_bytes(torrent.down_rate)} "/s"</span>
                                                        <span class="text-green-500">"↑ " {format_bytes(torrent.up_rate)} "/s"</span>
                                                    </div>
                                                    <div class="text-muted-foreground">{format_eta(torrent.eta)}</div>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                                <Show when=move || processed_torrents.get().is_empty() fallback=|| ()>
                                    <div class="p-12 text-center mt-10 text-muted-foreground">
                                        <div class="mb-4 text-6xl opacity-20">"📭"</div>
                                        "No torrents found."
                                    </div>
                                </Show>
                            </div>
                        }.into_view()
                    }}
                </div>
            </main>
            
            // MOBILE BOTTOM NAV
            <nav class="md:hidden fixed bottom-0 inset-x-0 backdrop-blur-xl border-t border-border pb-safe z-30 flex justify-between items-center px-6 py-2 bg-background/80">
                <button class={move || tab_btn_class("torrents")} on:click=move |_| set_active_tab.set("torrents")>
                    <svg class="w-6 h-6 mb-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" /></svg>
                    <span class="text-[10px] font-medium">"List"</span>
                </button>
                <button class={move || tab_btn_class("dashboard")} on:click=move |_| set_active_tab.set("dashboard")>
                    <svg class="w-6 h-6 mb-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
                    <span class="text-[10px] font-medium">"Dashboard"</span>
                </button>
                    <button class={move || tab_btn_class("settings")} on:click=move |_| set_active_tab.set("settings")>
                    <svg class="w-6 h-6 mb-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" /></svg>
                    <span class="text-[10px] font-medium">"Settings"</span>
                </button>
            </nav>
    
            // Modal (Dark backdrop always)
            <Show when=move || show_modal.get() fallback=|| ()>
                <div class="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-end md:items-center justify-center z-50 animate-in fade-in duration-200 sm:p-4">
                    <div class="bg-card p-6 rounded-t-2xl md:rounded-lg w-full max-w-lg shadow-xl border border-border ring-0 transform transition-all animate-in slide-in-from-bottom-10 md:slide-in-from-bottom-0 md:zoom-in-95">
                        <div class="flex justify-between items-center mb-6">
                            <h3 class="text-xl font-bold text-card-foreground">"Add New Torrent"</h3>
                            <button on:click=move |_| set_show_modal.set(false) class="p-1 hover:bg-accent rounded-full transition-colors text-muted-foreground">
                                <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                            </button>
                        </div>
                        <div class="relative mb-6">
                            <input 
                                type="text" 
                                class="w-full bg-input border border-input rounded-md p-3 pl-10 text-foreground focus:border-ring focus:ring-1 focus:ring-ring focus:outline-none transition-all placeholder:text-muted-foreground"
                                placeholder="Paste Magnet Link or URL"
                                on:input=move |ev| set_magnet_link.set(event_target_value(&ev))
                                prop:value=magnet_link
                                autoFocus
                            />
                            <div class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">
                                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" /></svg>
                            </div>
                        </div>
                        <div class="flex gap-3">
                            <Button class="flex-1" on_click=Callback::from(move |e| add_torrent(e))>
                                "Add Download"
                            </Button>
                        </div>
                    </div>
                </div>
            </Show>

            <ContextMenu
                position=cm_pos.get()
                visible=cm_visible.get()
                torrent_hash=cm_target_hash.get()
                on_close=Callback::from(move |_| set_cm_visible.set(false))
                on_action=Callback::from(move |(action, hash): (String, String)| {
                    logging::log!("App: Received action '{}' for hash '{}'", action, hash);
                    if action == "delete" || action == "delete_with_data" {
                        logging::log!("App: Showing delete modal");
                        set_pending_action.set(Some((action, hash)));
                        set_show_delete_modal.set(true);
                    } else {
                        // Execute immediately for start/stop
                        spawn_local(async move {
                            let body = serde_json::json!({
                                "hash": hash,
                                "action": action
                            });
                            let _ = gloo_net::http::Request::post("/api/torrents/action")
                                .header("Content-Type", "application/json")
                                .body(body.to_string())
                                .unwrap()
                                .send()
                                .await;
                        });
                    }
                })
            />

            // Delete Confirmation Modal
            <Modal
                title="Confirm Deletion"
                visible=show_delete_modal
                is_danger=true
                confirm_text="Delete Forever"
                on_cancel=Callback::from(move |_| {
                    set_show_delete_modal.set(false);
                    set_pending_action.set(None);
                })
                on_confirm=Callback::from(move |_| {
                    if let Some((action, hash)) = pending_action.get() {
                        spawn_local(async move {
                            let body = serde_json::json!({
                                "hash": hash,
                                "action": action
                            });
                            let _ = gloo_net::http::Request::post("/api/torrents/action")
                                .header("Content-Type", "application/json")
                                .body(body.to_string())
                                .unwrap()
                                .send()
                                .await;
                        });
                    }
                    set_show_delete_modal.set(false);
                    set_pending_action.set(None);
                })
            >
                <p>"Are you definitely sure you want to delete this torrent?"</p>
                <Show when=move || pending_action.get().map(|(a, _)| a == "delete_with_data").unwrap_or(false)>
                    <p class="mt-2 text-destructive font-bold">"⚠️ This will also permanently delete the downloaded files from the disk."</p>
                </Show>
            </Modal>
        </div>
    }
}
