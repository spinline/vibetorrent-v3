use leptos::*;
use thaw::*;
use shared::{Torrent, AppEvent, TorrentStatus, Theme};
use crate::components::toolbar::Toolbar;
use crate::components::sidebar::Sidebar;
// use crate::components::context_menu::ContextMenu;
// use crate::components::modal::Modal;
use crate::components::status_bar::StatusBar;
use crate::components::torrent_table::TorrentTable;
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

    // Persist Theme Logic
    create_effect(move |_| {
         let val = match theme.get() {
             Theme::Midnight => "Midnight",
             Theme::Light => "Light",
             Theme::Amoled => "Amoled",
         };
         
         if let Some(doc) = window().document() {
             if let Some(body) = doc.body() {
                 let list = body.class_list();
                 // Reset classes
                 let _ = list.remove_1("dark");
                 let _ = list.remove_1("amoled");

                 match theme.get() {
                     Theme::Light => {},
                     Theme::Midnight => { let _ = list.add_1("dark"); },
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

    // Remove Loading Spinner
    create_effect(move |_| {
        if let Some(doc) = window().document() {
            if let Some(el) = doc.get_element_by_id("app-loading") {
                el.remove();
            }
        }
    });

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
            let mut es = EventSource::new("/api/events").unwrap();
            let mut stream = es.subscribe("message").unwrap();
            
            loop {
                match stream.next().await {
                    Some(Ok((_, msg))) => {
                        let data = msg.data().as_string().unwrap();
                        if let Ok(event) = serde_json::from_str::<AppEvent>(&data) {
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
                    }
                    None => break,
                    _ => {}
                }
            }
        });
    });

    // Toolbar Callbacks
    let on_add = Callback::new(move |_| set_show_modal.set(true));
    let on_start = Callback::new(move |_| logging::log!("Start all - to be implemented with selection"));
    let on_pause = Callback::new(move |_| logging::log!("Pause all - to be implemented with selection"));
    let on_delete = Callback::new(move |_| logging::log!("Delete - to be implemented with selection"));
    let on_settings = Callback::new(move |_| set_active_tab.set(if active_tab.get() == "settings" { "torrents" } else { "settings" }));

    view! {
        <div class="flex h-screen overflow-hidden bg-background text-foreground font-sans">
            <Sidebar 
                active_filter=filter_status 
                on_filter_change=Callback::new(move |s| set_filter_status.set(s))
            />
            <div class="flex-1 flex flex-col min-w-0">
                <Toolbar 
                    on_add=on_add
                    on_start=on_start
                    on_pause=on_pause
                    on_delete=on_delete
                    on_settings=on_settings
                />
                
                {move || if active_tab.get() == "settings" {
                    view! {
                        <div class="p-6">
                            <h1 class="text-2xl font-bold mb-4">"Settings"</h1>
                            <div class="flex gap-4">
                                <Button on_click=move |_| set_theme.set(Theme::Midnight)>"Midnight"</Button>
                                <Button on_click=move |_| set_theme.set(Theme::Light)>"Light"</Button>
                                <Button on_click=move |_| set_theme.set(Theme::Amoled)>"Amoled"</Button>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <TorrentTable torrents=processed_torrents /> }.into_view()
                }}

                <StatusBar />
            </div>

            // Add Torrent Modal (Inlined)
            <Show when=move || show_modal.get() fallback=|| ()>
                <div class="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-end md:items-center justify-center z-[200] animate-in fade-in duration-200 sm:p-4">
                    <div class="bg-card p-6 rounded-t-2xl md:rounded-lg w-full max-w-sm shadow-xl border border-border ring-0 transform transition-all animate-in slide-in-from-bottom-10 md:slide-in-from-bottom-0 md:zoom-in-95">
                        <h3 class="text-lg font-semibold text-card-foreground mb-4">"Add Torrent"</h3>
                        
                        <div class="space-y-4 mb-6">
                             <p class="text-sm text-muted-foreground">"Paste a magnet link or URL to start downloading."</p>
                             <input 
                                 type="text" 
                                 class="w-full bg-input border border-input rounded-md p-2 text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                                 placeholder="magnet:?xt=urn:btih:..."
                                 on:input=move |ev| set_magnet_link.set(event_target_value(&ev))
                                 prop:value=magnet_link
                                 autoFocus
                             />
                        </div>
                        
                        <div class="flex justify-end gap-3">
                            <button 
                                class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2"
                                on:click=move |_| set_show_modal.set(false)
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
                                on:click=move |_| add_torrent(())
                            >
                                "Add Download"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
