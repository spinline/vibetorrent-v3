use leptos::*;
use shared::{Torrent, AppEvent, TorrentStatus, Theme};
use crate::components::context_menu::ContextMenu;
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
    create_effect(move |_| {
         let val = match theme.get() {
             Theme::Midnight => "Midnight",
             Theme::Light => "Light",
             Theme::Amoled => "Amoled",
         };
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

    // Context Menu Signals
    let (cm_visible, set_cm_visible) = create_signal(false);
    let (cm_pos, set_cm_pos) = create_signal((0, 0));
    let (cm_target_hash, set_cm_target_hash) = create_signal(String::new());
    
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
                                                 if let Some(v) = diff.down_rate { target.down_rate = v; }
                                                 if let Some(v) = diff.up_rate { target.up_rate = v; }
                                                 if let Some(v) = diff.percent_complete { target.percent_complete = v; }
                                                 if let Some(v) = diff.completed { target.completed = v; }
                                                 if let Some(v) = diff.eta { target.eta = v; }
                                                 if let Some(v) = diff.status { target.status = v; }
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
    let get_theme_classes = move || {
        match theme.get() {
            Theme::Midnight => (
                "bg-[#0a0a0c] text-white selection:bg-blue-500/30", // Main bg
                "bg-[#111116]/80 backdrop-blur-xl border-white/5", // Sidebar
                "bg-[#111116] border-white/5 shadow-2xl", // Card/Table bg
                "text-gray-200", // Primary Text
                "text-gray-400", // Secondary Text
                "hover:bg-white/5", // Hover
                "border-white/5" // Border
            ),
            Theme::Light => (
                "bg-gray-50 text-gray-900 selection:bg-blue-500/20",
                "bg-white/80 backdrop-blur-xl border-gray-200",
                "bg-white border-gray-200 shadow-xl",
                "text-gray-900",
                "text-gray-500",
                "hover:bg-gray-100",
                "border-gray-200"
            ),
            Theme::Amoled => (
                "bg-black text-white selection:bg-blue-600/40",
                "bg-black border-gray-800",
                "bg-black border-gray-800",
                "text-gray-200",
                "text-gray-500",
                "hover:bg-gray-900",
                "border-gray-800"
            ),
        }
    };

    let filter_btn_class = move |status: Option<TorrentStatus>| {
        let (_base_bg, _, _, _, text_sec, hover, _) = get_theme_classes();
        let base = "block px-4 py-2 rounded-xl transition-all duration-200 text-left w-full flex items-center gap-3 border";
        let active = filter_status.get() == status;
        if active {
             format!("{} bg-blue-600/20 text-blue-500 border-blue-500/30 font-medium", base)
        } else {
             format!("{} {} {} border-transparent hover:text-gray-300", base, hover, text_sec)
        }
    };
    
    let tab_btn_class = move |tab: &str| {
        let active = active_tab.get() == tab;
        let base = "flex flex-col items-center justify-center p-2 flex-1 transition-colors relative";
        if active {
            format!("{} text-blue-500", base)
        } else {
            "flex flex-col items-center justify-center p-2 flex-1 transition-colors relative text-gray-400 hover:text-gray-300".to_string()
        }
    };
    
    // Sidebar Content Logic
    let sidebar_content = move || {
        let (_, _, _, _text_pri, text_sec, _, border) = get_theme_classes();
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
            
            <div class={format!("text-xs font-bold uppercase tracking-widest mb-4 px-2 {}", text_sec)}>"Filters"</div>
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
            
            <div class={format!("mt-auto pt-6 border-t {}", border)}>
                <div class={format!("rounded-xl p-4 border relative overflow-hidden {}", border)}>
                    <div class={format!("absolute inset-0 opacity-5 {}", if theme.get() == Theme::Light { "bg-black" } else { "bg-white" })}></div>
                    <div class={format!("text-xs mb-2 z-10 relative {}", text_sec)}>"Storage"</div>
                    <div class="w-full bg-gray-500/20 rounded-full h-1.5 mb-2 overflow-hidden z-10 relative">
                         <div class="bg-gradient-to-r from-blue-500 to-purple-500 w-[70%] h-full rounded-full"></div>
                    </div>
                    <div class={format!("flex justify-between text-xs z-10 relative {}", text_sec)}>
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
        {move || {
            let (main_bg, sidebar_bg, card_bg, text_pri, text_sec, hover, border) = get_theme_classes();
            
            view! {
                <div class={format!("min-h-screen font-sans flex flex-col md:flex-row overflow-hidden transition-colors duration-300 {}", main_bg)}>
                    // DESKTOP SIDEBAR
                    <aside class={format!("hidden md:flex flex-col w-72 border-r p-6 z-20 h-screen {}", sidebar_bg)}>
                        {sidebar_content}
                    </aside>
                    
                    // MOBILE SIDEBAR
                    <div 
                        class={move || if show_mobile_sidebar.get() { "fixed inset-0 z-50 flex md:hidden" } else { "hidden" }}
                        on:click=move |_| set_show_mobile_sidebar.set(false)
                    >
                        <div class="fixed inset-0 bg-black/60 backdrop-blur-sm transition-opacity"></div>
                        <aside 
                            class={format!("relative w-80 max-w-[85vw] h-full shadow-2xl p-6 flex flex-col animate-in slide-in-from-left duration-300 border-r {}", sidebar_bg)}
                            on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                        >
                            <button class={format!("absolute top-4 right-4 p-2 hover:opacity-80 {}", text_sec)} on:click=move |_| set_show_mobile_sidebar.set(false)>
                                <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                            </button>
                            {sidebar_content}
                        </aside>
                    </div>

                    // MAIN CONTENT
                    <main class="flex-1 h-screen overflow-y-auto overflow-x-hidden relative pb-24 md:pb-0">
                        <header class={format!("sticky top-0 z-10 border-b px-6 py-4 flex justify-between items-center {}", sidebar_bg)}>
                            <div class="flex items-center gap-3">
                                 <button class={format!("md:hidden p-1 -ml-2 hover:opacity-80 {}", text_sec)} on:click=move |_| set_show_mobile_sidebar.set(true)>
                                    <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" /></svg>
                                 </button>
                                <h2 class={format!("text-xl font-bold flex items-center gap-2 {}", text_pri)}>
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
                                <div class={format!("hidden md:block text-xs font-mono {}", text_sec)}>
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
                                <button 
                                    class="px-5 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-xl hover:shadow-lg hover:shadow-blue-500/30 hover:scale-105 active:scale-95 transition-all text-sm font-bold text-white flex items-center gap-2"
                                    on:click=move |_| set_show_modal.set(true)
                                >
                                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
                                    <span class="hidden md:inline">"Add Torrent"</span>
                                    <span class="md:hidden">"Add"</span>
                                </button>
                                
                                <button 
                                    class={format!("hidden md:flex p-2.5 rounded-xl hover:bg-white/5 active:scale-95 transition-all text-gray-400 hover:text-white border border-transparent hover:border-white/10 {}", if active_tab.get() == "settings" { "bg-blue-500/10 text-blue-500 border-blue-500/20" } else { "" })}
                                    on:click=move |_| set_active_tab.set(if active_tab.get() == "settings" { "torrents" } else { "settings" })
                                    title="Settings"
                                >
                                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                    </svg>
                                </button>
                            </div>
                        </header>

                        <div class="p-6 max-w-7xl mx-auto animate-in fade-in duration-500">
                            {move || if active_tab.get() == "settings" {
                                view! {
                                    <div class="space-y-8">
                                        <div>
                                            <h3 class={format!("text-lg font-bold mb-4 {}", text_pri)}>"Appearance"</h3>
                                            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                                                {theme_option(Theme::Midnight, "Midnight", "bg-[#0a0a0c] border border-gray-700")}
                                                {theme_option(Theme::Light, "Light", "bg-gray-100 border border-gray-300")}
                                                {theme_option(Theme::Amoled, "Amoled", "bg-black border border-gray-800")}
                                            </div>
                                        </div>
                                        <div class={format!("p-6 rounded-2xl border {}", card_bg)}>
                                            <h3 class={format!("text-lg font-bold mb-2 {}", text_pri)}>"About VibeTorrent"</h3>
                                            <p class={format!("text-sm {}", text_sec)}>"Version 3.0.0 (Rust + WebAssembly)"</p>
                                        </div>
                                    </div>
                                }.into_view()
                            } else if active_tab.get() == "dashboard" {
                                view! { 
                                     <div class="text-center py-20 opacity-50">"Dashboard Charts Coming Soon..."</div>
                                }.into_view()
                            } else {
                                view! {
                                    // Torrent List (Desktop)
                                    <div class={format!("hidden md:block rounded-2xl border shadow-sm overflow-hidden {}", card_bg)}>
                                        <table class="w-full text-left table-fixed">
                                            <thead class={format!("uppercase text-xs font-bold tracking-wider {}", text_sec)}>
                                                <tr>
                                                    <th class="px-6 py-4 cursor-pointer hover:opacity-80" on:click=move |_| sort(0)>"Name"</th>
                                                    <th class="px-6 py-4 cursor-pointer hover:opacity-80 w-28 text-right whitespace-nowrap" on:click=move |_| sort(1)>"Size"</th>
                                                    <th class="px-6 py-4 cursor-pointer hover:opacity-80 w-36" on:click=move |_| sort(2)>"Progress"</th>
                                                    <th class="px-6 py-4 cursor-pointer hover:opacity-80 w-28 text-right whitespace-nowrap" on:click=move |_| sort(3)>"Down"</th>
                                                    <th class="px-6 py-4 cursor-pointer hover:opacity-80 w-28 text-right whitespace-nowrap" on:click=move |_| sort(4)>"Up"</th>
                                                    <th class="px-6 py-4 cursor-pointer hover:opacity-80 w-28 text-right whitespace-nowrap" on:click=move |_| sort(5)>"ETA"</th>
                                                    <th class="px-6 py-4 text-center w-28">"Status"</th>
                                                </tr>
                                            </thead>
                                            <tbody class={format!("divide-y {}", border)}>
                                                <For
                                                    each=move || processed_torrents.get()
                                                    key=|t| format!("{}-{}-{}-{}-{}-{}", t.hash, t.down_rate, t.up_rate, t.percent_complete, t.eta, t.error_message)
                                                    children=move |torrent| {
                                                        let status_color = match torrent.status {
                                                            TorrentStatus::Downloading => "text-blue-500 bg-blue-500/10 border-blue-500/20",
                                                            TorrentStatus::Seeding => "text-green-500 bg-green-500/10 border-green-500/20",
                                                            TorrentStatus::Paused => "text-yellow-500 bg-yellow-500/10 border-yellow-500/20",
                                                            TorrentStatus::Error => "text-red-500 bg-red-500/10 border-red-500/20",
                                                            _ => "text-gray-400 bg-gray-500/10"
                                                        };
                                                        let status_text = format!("{:?}", torrent.status);
                                                        let error_msg = torrent.error_message.clone();
                                                        let error_msg_view = error_msg.clone();
                                                        
                                                        view! {
                                                            <tr 
                                                                class={format!("transition-colors group {}", hover)}
                                                                on:contextmenu=move |e: web_sys::MouseEvent| {
                                                                    e.prevent_default();
                                                                    set_cm_pos.set((e.client_x(), e.client_y()));
                                                                    set_cm_target_hash.set(torrent.hash.clone());
                                                                    set_cm_visible.set(true);
                                                                }
                                                            >
                                                                <td class="px-6 py-4 max-w-sm">
                                                                    <div class={format!("font-medium truncate transition-colors {}", text_pri)} title={torrent.name.clone()}>
                                                                        {torrent.name}
                                                                    </div>
                                                                    <Show when=move || !error_msg.is_empty() fallback=|| ()>
                                                                        <div class="text-xs text-red-500 mt-1">{error_msg_view.clone()}</div>
                                                                    </Show>
                                                                </td>
                                                                <td class={format!("px-6 py-4 text-sm font-mono text-right whitespace-nowrap {}", text_sec)}>{format_bytes(torrent.size)}</td>
                                                                <td class="px-6 py-4">
                                                                    <div class="flex flex-col gap-1.5">
                                                                        <div class={format!("flex justify-between text-xs {}", text_sec)}>
                                                                            <span>{format!("{:.1}%", torrent.percent_complete)}</span>
                                                                        </div>
                                                                        <div class="w-full bg-gray-500/20 rounded-full h-1.5 overflow-hidden">
                                                                            <div 
                                                                                class="bg-blue-500 h-full rounded-full transition-all duration-500" 
                                                                                style=format!("width: {}%", torrent.percent_complete)
                                                                            ></div>
                                                                        </div>
                                                                    </div>
                                                                </td>
                                                                <td class={format!("px-6 py-4 font-mono text-xs text-right whitespace-nowrap {}", text_sec)}>
                                                                    {if torrent.down_rate > 0 {
                                                                        view! { <span class="text-blue-500">{format_bytes(torrent.down_rate)} "/s"</span> }.into_view()
                                                                    } else {
                                                                        view! { <span class="text-gray-600">"-"</span> }.into_view()
                                                                    }}
                                                                </td>
                                                                <td class={format!("px-6 py-4 font-mono text-xs text-right whitespace-nowrap {}", text_sec)}>
                                                                    {if torrent.up_rate > 0 {
                                                                        view! { <span class="text-green-500">{format_bytes(torrent.up_rate)} "/s"</span> }.into_view()
                                                                    } else {
                                                                        view! { <span class="text-gray-600">"-"</span> }.into_view()
                                                                    }}
                                                                </td>
                                                                <td class={format!("px-6 py-4 text-xs font-mono text-right whitespace-nowrap {}", text_sec)}>
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
                                            key=|t| format!("{}-{}-{}-{}-{}-{}", t.hash, t.down_rate, t.up_rate, t.percent_complete, t.eta, t.error_message)
                                            children=move |torrent| {
                                                let status_color = match torrent.status {
                                                    TorrentStatus::Downloading => "text-blue-500",
                                                    TorrentStatus::Seeding => "text-green-500",
                                                    TorrentStatus::Paused => "text-yellow-500",
                                                    TorrentStatus::Error => "text-red-500",
                                                    _ => "text-gray-400"
                                                };
                                                
                                                view! {
                                                    <div class={format!("rounded-2xl p-4 border shadow-sm active:scale-[0.98] transition-transform {}", card_bg)}>
                                                        <div class="flex justify-between items-start mb-3">
                                                            <div class={format!("font-medium line-clamp-2 pr-4 {}", text_pri)}>{torrent.name}</div>
                                                            <div class={format!("text-xs font-bold {}", status_color)}>
                                                                {format!("{:?}", torrent.status)}
                                                            </div>
                                                        </div>
                                                        <div class="mb-4">
                                                            <div class={format!("flex justify-between text-xs mb-1 {}", text_sec)}>
                                                                <span>{format_bytes(torrent.size)}</span>
                                                                <span>{format!("{:.1}%", torrent.percent_complete)}</span>
                                                            </div>
                                                            <div class="w-full bg-gray-500/20 rounded-full h-1.5 overflow-hidden">
                                                                <div 
                                                                    class="bg-blue-500 h-full rounded-full transition-all duration-500" 
                                                                    style=format!("width: {}%", torrent.percent_complete)
                                                                ></div>
                                                            </div>
                                                        </div>
                                                        <div class="flex justify-between items-center text-xs font-mono opacity-80">
                                                            <div class="flex gap-3">
                                                                <span class="text-blue-500">"↓ " {format_bytes(torrent.down_rate)} "/s"</span>
                                                                <span class="text-green-500">"↑ " {format_bytes(torrent.up_rate)} "/s"</span>
                                                            </div>
                                                            <div class={text_sec}>{format_eta(torrent.eta)}</div>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                        <Show when=move || processed_torrents.get().is_empty() fallback=|| ()>
                                            <div class={format!("p-12 text-center mt-10 {}", text_sec)}>
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
                    <nav class={format!("md:hidden fixed bottom-0 inset-x-0 backdrop-blur-xl border-t pb-safe z-30 flex justify-between items-center px-6 py-2 {}", sidebar_bg)}>
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
                        <div class="fixed inset-0 bg-black/80 backdrop-blur-md flex items-end md:items-center justify-center z-50 animate-in fade-in duration-200 sm:p-4">
                            <div class="bg-[#16161c] p-6 rounded-t-2xl md:rounded-2xl w-full max-w-lg shadow-2xl border border-white/10 ring-1 ring-white/5 transform transition-all animate-in slide-in-from-bottom-10 md:slide-in-from-bottom-0 md:zoom-in-95">
                                <div class="flex justify-between items-center mb-6">
                                    <h3 class="text-xl font-bold text-white">"Add New Torrent"</h3>
                                    <button on:click=move |_| set_show_modal.set(false) class="p-1 hover:bg-white/10 rounded-full transition-colors">
                                        <svg class="w-6 h-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                                    </button>
                                </div>
                                <div class="relative mb-6">
                                    <input 
                                        type="text" 
                                        class="w-full bg-black/30 border border-white/10 rounded-xl p-4 pl-12 text-white focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none transition-all placeholder:text-gray-600"
                                        placeholder="Paste Magnet Link or URL"
                                        on:input=move |ev| set_magnet_link.set(event_target_value(&ev))
                                        prop:value=magnet_link
                                        autoFocus
                                    />
                                    <div class="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500">
                                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" /></svg>
                                    </div>
                                </div>
                                <div class="flex gap-3">
                                    <button class="flex-1 px-4 py-3.5 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-xl hover:shadow-[0_0_20px_rgba(59,130,246,0.3)] transition-all font-bold text-white shadow-lg active:scale-95" on:click=add_torrent>
                                        "Add Download"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </Show>

                    <ContextMenu
                        position=cm_pos.get()
                        visible=cm_visible.get()
                        torrent_hash=cm_target_hash.get()
                        on_close=Callback::from(move |_| set_cm_visible.set(false))
                    />
                </div>
            }
        }}
    }
}
