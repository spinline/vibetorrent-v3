use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::store::{get_action_messages, show_toast_with_signal};
use crate::api;
use shared::NotificationLevel;
use crate::components::context_menu::TorrentContextMenu;
use leptos_shadcn_card::{Card, CardHeader, CardTitle, CardContent};

fn format_bytes(bytes: i64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes < 1024 { return format!("{} B", bytes); }
    let i = (bytes as f64).log2().div_euclid(10.0) as usize;
    format!("{:.1} {}", (bytes as f64) / 1024_f64.powi(i as i32), UNITS[i])
}

fn format_speed(bytes_per_sec: i64) -> String {
    if bytes_per_sec == 0 { return "0 B/s".to_string(); }
    format!("{}/s", format_bytes(bytes_per_sec))
}

fn format_duration(seconds: i64) -> String {
    if seconds <= 0 { return "∞".to_string(); }
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    if days > 0 { format!("{}d {}h", days, hours) }
    else if hours > 0 { format!("{}h {}m", hours, minutes) }
    else if minutes > 0 { format!("{}m {}s", minutes, secs) }
    else { format!("{}s", secs) }
}

fn format_date(timestamp: i64) -> String {
    if timestamp <= 0 { return "N/A".to_string(); }
    let dt = chrono::DateTime::from_timestamp(timestamp, 0);
    match dt { Some(dt) => dt.format("%d/%m/%Y %H:%M").to_string(), None => "N/A".to_string() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortColumn {
    Name, Size, Progress, Status, DownSpeed, UpSpeed, ETA, AddedDate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortDirection { Ascending, Descending }

#[component]
pub fn TorrentTable() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let sort_col = signal(SortColumn::AddedDate);
    let sort_dir = signal(SortDirection::Descending);

    let filtered_hashes = Memo::new(move |_| {
        let torrents_map = store.torrents.get();
        let filter = store.filter.get();
        let search = store.search_query.get();
        let search_lower = search.to_lowercase();
        
        let mut torrents: Vec<shared::Torrent> = torrents_map.values().filter(|t| {
            let matches_filter = match filter {
                crate::store::FilterStatus::All => true,
                crate::store::FilterStatus::Downloading => t.status == shared::TorrentStatus::Downloading,
                crate::store::FilterStatus::Seeding => t.status == shared::TorrentStatus::Seeding,
                crate::store::FilterStatus::Completed => t.status == shared::TorrentStatus::Seeding || (t.status == shared::TorrentStatus::Paused && t.percent_complete >= 100.0),
                crate::store::FilterStatus::Paused => t.status == shared::TorrentStatus::Paused,
                crate::store::FilterStatus::Inactive => t.status == shared::TorrentStatus::Paused || t.status == shared::TorrentStatus::Error,
                _ => true,
            };
            let matches_search = if search_lower.is_empty() { true } else { t.name.to_lowercase().contains(&search_lower) };
            matches_filter && matches_search
        }).cloned().collect();

        torrents.sort_by(|a, b| {
            let col = sort_col.0.get();
            let dir = sort_dir.0.get();
            let cmp = match col {
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Size => a.size.cmp(&b.size),
                SortColumn::Progress => a.percent_complete.partial_cmp(&b.percent_complete).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Status => format!("{:?}", a.status).cmp(&format!("{:?}", b.status)),
                SortColumn::DownSpeed => a.down_rate.cmp(&b.down_rate),
                SortColumn::UpSpeed => a.up_rate.cmp(&b.up_rate),
                SortColumn::ETA => {
                    let a_eta = if a.eta <= 0 { i64::MAX } else { a.eta };
                    let b_eta = if b.eta <= 0 { i64::MAX } else { b.eta };
                    a_eta.cmp(&b_eta)
                }
                SortColumn::AddedDate => a.added_date.cmp(&b.added_date),
            };
            if dir == SortDirection::Descending { cmp.reverse() } else { cmp }
        });
        torrents.into_iter().map(|t| t.hash.clone()).collect::<Vec<String>>()
    });

    let handle_sort = move |col: SortColumn| {
        if sort_col.0.get() == col {
            sort_dir.1.update(|d| {
                *d = match d { SortDirection::Ascending => SortDirection::Descending, SortDirection::Descending => SortDirection::Ascending };
            });
        } else {
            sort_col.1.set(col);
            sort_dir.1.set(SortDirection::Ascending);
        }
    };

    let sort_arrow = move |col: SortColumn| {
        if sort_col.0.get() == col {
            match sort_dir.0.get() {
                SortDirection::Ascending => view! { <span class="ml-1 text-xs">"▲"</span> }.into_any(),
                SortDirection::Descending => view! { <span class="ml-1 text-xs">"▼"</span> }.into_any(),
            }
        } else { view! { <span class="ml-1 text-xs opacity-0 group-hover:opacity-50">"▲"</span> }.into_any() }
    };

    let on_action = Callback::new(move |(action, hash): (String, String)| {
        let (success_msg_str, error_msg_str): (&'static str, &'static str) = get_action_messages(&action);
        let success_msg = success_msg_str.to_string();
        let error_msg = error_msg_str.to_string();
        let notifications = store.notifications;
        spawn_local(async move {
            let result = match action.as_str() {
                "delete" => api::torrent::delete(&hash).await,
                "delete_with_data" => api::torrent::delete_with_data(&hash).await,
                "start" => api::torrent::start(&hash).await,
                "stop" => api::torrent::stop(&hash).await,
                _ => api::torrent::action(&hash, &action).await,
            };
            match result {
                Ok(_) => show_toast_with_signal(notifications, NotificationLevel::Success, success_msg),
                Err(e) => show_toast_with_signal(notifications, NotificationLevel::Error, format!("{}: {:?}", error_msg, e)),
            }
        });
    });

    view! {
        <div class="h-full bg-background relative flex flex-col overflow-hidden">
            // --- DESKTOP VIEW ---
            <div class="hidden md:flex flex-col h-full overflow-hidden">
                // Header
                <div class="flex items-center text-xs uppercase text-muted-foreground border-b border-border bg-muted/50 h-9 shrink-0 px-2 font-medium">
                    <div class="flex-1 px-2 cursor-pointer hover:text-foreground group select-none flex items-center" on:click=move |_| handle_sort(SortColumn::Name)>
                        "Name" {move || sort_arrow(SortColumn::Name)}
                    </div>
                    <div class="w-24 px-2 cursor-pointer hover:text-foreground group select-none flex items-center" on:click=move |_| handle_sort(SortColumn::Size)>
                        "Size" {move || sort_arrow(SortColumn::Size)}
                    </div>
                    <div class="w-48 px-2 cursor-pointer hover:text-foreground group select-none flex items-center" on:click=move |_| handle_sort(SortColumn::Progress)>
                        "Progress" {move || sort_arrow(SortColumn::Progress)}
                    </div>
                    <div class="w-24 px-2 cursor-pointer hover:text-foreground group select-none flex items-center" on:click=move |_| handle_sort(SortColumn::Status)>
                        "Status" {move || sort_arrow(SortColumn::Status)}
                    </div>
                    <div class="w-24 px-2 cursor-pointer hover:text-foreground group select-none flex items-center" on:click=move |_| handle_sort(SortColumn::DownSpeed)>
                        "DL Speed" {move || sort_arrow(SortColumn::DownSpeed)}
                    </div>
                    <div class="w-24 px-2 cursor-pointer hover:text-foreground group select-none flex items-center" on:click=move |_| handle_sort(SortColumn::UpSpeed)>
                        "Up Speed" {move || sort_arrow(SortColumn::UpSpeed)}
                    </div>
                    <div class="w-24 px-2 cursor-pointer hover:text-foreground group select-none flex items-center" on:click=move |_| handle_sort(SortColumn::ETA)>
                        "ETA" {move || sort_arrow(SortColumn::ETA)}
                    </div>
                    <div class="w-32 px-2 cursor-pointer hover:text-foreground group select-none flex items-center" on:click=move |_| handle_sort(SortColumn::AddedDate)>
                        "Date" {move || sort_arrow(SortColumn::AddedDate)}
                    </div>
                </div>

                // Regular List
                <div class="flex-1 overflow-y-auto min-h-0">
                    <For each=move || filtered_hashes.get() key=|hash| hash.clone() children={
                        let on_action = on_action.clone();
                        move |hash| {
                            let h = hash.clone();
                            view! { 
                                <TorrentContextMenu torrent_hash=h on_action=on_action.clone()>
                                    <TorrentRow hash=hash.clone() /> 
                                </TorrentContextMenu>
                            }
                        }
                    } />
                </div>
            </div>

            // --- MOBILE VIEW ---
            <div class="md:hidden flex flex-col h-full bg-muted/10 relative overflow-hidden">
                <div class="flex-1 overflow-y-auto p-3 min-h-0">
                    <For each=move || filtered_hashes.get() key=|hash| hash.clone() children={
                        let on_action = on_action.clone();
                        move |hash| {
                            let h = hash.clone();
                            view! { 
                                 <div class="pb-3">
                                    <TorrentContextMenu torrent_hash=h on_action=on_action.clone()>
                                        <TorrentCard hash=hash.clone() /> 
                                    </TorrentContextMenu>
                                </div>
                            }
                        }
                    } />
                </div>
            </div>
        </div>
    }
}

#[component]
fn TorrentRow(
    hash: String,
) -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let h = hash.clone();
    let torrent = Memo::new(move |_| store.torrents.with(|map| map.get(&h).cloned()));

    let stored_hash = StoredValue::new(hash.clone());

    view! {
        <Show when=move || torrent.get().is_some() fallback=|| ()>
            {
                move || {
                    let t = torrent.get().unwrap();
                    let t_name = t.name.clone();
                    let status_color = match t.status { shared::TorrentStatus::Seeding => "text-green-500", shared::TorrentStatus::Downloading => "text-blue-500", shared::TorrentStatus::Paused => "text-yellow-500", shared::TorrentStatus::Error => "text-red-500", _ => "text-muted-foreground" };
                    
                    view! {
                        <div 
                            class=move || {
                                let selected = store.selected_torrent.get();
                                let is_selected = selected.as_deref() == Some(stored_hash.get_value().as_str());
                                if is_selected {
                                    "flex items-center text-sm bg-primary/10 border-b border-border h-[48px] px-2 select-none cursor-pointer transition-colors w-full"
                                } else {
                                    "flex items-center text-sm hover:bg-muted/50 border-b border-border h-[48px] px-2 select-none cursor-pointer transition-colors w-full"
                                }
                            }
                            on:click=move |_| store.selected_torrent.set(Some(stored_hash.get_value()))
                        >
                            <div class="flex-1 min-w-0 px-2 font-medium truncate" title=t_name.clone()>{t_name.clone()}</div>
                            <div class="w-24 px-2 font-mono text-xs text-muted-foreground">{format_bytes(t.size)}</div>
                            <div class="w-48 px-2">
                                <div class="flex items-center gap-2">
                                    <div class="h-2 w-full bg-secondary rounded-full overflow-hidden">
                                        <div class="h-full bg-primary transition-all duration-500" style=format!("width: {}%", t.percent_complete)></div>
                                    </div>
                                    <span class="text-[10px] text-muted-foreground w-10 text-right">{format!("{:.1}%", t.percent_complete)}</span>
                                </div>
                            </div>
                            <div class={format!("w-24 px-2 text-xs font-medium {}", status_color)}>{format!("{:?}", t.status)}</div>
                            <div class="w-24 px-2 text-right font-mono text-xs text-green-600 dark:text-green-500">{format_speed(t.down_rate)}</div>
                            <div class="w-24 px-2 text-right font-mono text-xs text-blue-600 dark:text-blue-500">{format_speed(t.up_rate)}</div>
                            <div class="w-24 px-2 text-right font-mono text-xs text-muted-foreground">{format_duration(t.eta)}</div>
                            <div class="w-32 px-2 text-right font-mono text-xs text-muted-foreground">{format_date(t.added_date)}</div>
                        </div>
                    }
                }
            }
        </Show>
    }
}

#[component]
fn TorrentCard(
    hash: String,
) -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let h = hash.clone();
    let torrent = Memo::new(move |_| store.torrents.with(|map| map.get(&h).cloned()));

    let stored_hash = StoredValue::new(hash.clone());

    view! {
        <Show when=move || torrent.get().is_some() fallback=|| ()>
            {
                move || {
                    let t = torrent.get().unwrap();
                    let t_name = t.name.clone();
                    let status_badge_class = match t.status { shared::TorrentStatus::Seeding => "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400 border-green-200 dark:border-green-800", shared::TorrentStatus::Downloading => "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400 border-blue-200 dark:border-blue-800", shared::TorrentStatus::Paused => "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400 border-yellow-200 dark:border-yellow-800", shared::TorrentStatus::Error => "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400 border-red-200 dark:border-red-800", _ => "bg-muted text-muted-foreground" };

                    view! {
                        <div
                            class=move || {
                                let selected = store.selected_torrent.get();
                                let is_selected = selected.as_deref() == Some(stored_hash.get_value().as_str());
                                if is_selected {
                                    "ring-2 ring-primary rounded-lg transition-all"
                                } else {
                                    "transition-all"
                                }
                            }
                            on:click=move |_| store.selected_torrent.set(Some(stored_hash.get_value()))
                        >
                        <Card class="h-full select-none cursor-pointer hover:border-primary transition-colors">
                            <CardHeader class="p-3 pb-0">
                                <div class="flex justify-between items-start gap-2">
                                    <CardTitle class="text-sm font-medium leading-tight line-clamp-2">{t_name.clone()}</CardTitle>
                                    <div class={format!("inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 {}", status_badge_class)}>{format!("{:?}", t.status)}</div>
                                </div>
                            </CardHeader>
                            <CardContent class="p-3 pt-2 gap-3 flex flex-col">
                                <div class="flex flex-col gap-1">
                                    <div class="flex justify-between text-[10px] text-muted-foreground">
                                        <span>{format_bytes(t.size)}</span>
                                        <span>{format!("{:.1}%", t.percent_complete)}</span>
                                    </div>
                                    <div class="h-1.5 w-full bg-secondary rounded-full overflow-hidden">
                                        <div class="h-full bg-primary transition-all duration-500" style=format!("width: {}%", t.percent_complete)></div>
                                    </div>
                                </div>
                                <div class="grid grid-cols-4 gap-2 text-[10px] font-mono text-muted-foreground pt-1 border-t border-border/50">
                                    <div class="flex flex-col text-blue-600 dark:text-blue-500"><span>"DL"</span><span>{format_speed(t.down_rate)}</span></div>
                                    <div class="flex flex-col text-green-600 dark:text-green-500"><span>"UP"</span><span>{format_speed(t.up_rate)}</span></div>
                                    <div class="flex flex-col"><span>"ETA"</span><span>{format_duration(t.eta)}</span></div>
                                    <div class="flex flex-col text-right"><span>"DATE"</span><span>{format_date(t.added_date)}</span></div>
                                </div>
                            </CardContent>
                        </Card>
                        </div>
                    }
                }
            }
        </Show>
    }
}
