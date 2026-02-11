use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashSet;
use icons::{ArrowUpDown};
use crate::store::{get_action_messages, show_toast};
use crate::api;
use shared::NotificationLevel;
use crate::components::context_menu::TorrentContextMenu;
use crate::components::ui::card::{Card, CardHeader, CardTitle, CardContent as CardBody};
use crate::components::ui::data_table::*;
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::button::{Button, ButtonVariant};

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
    
    // Multi-selection state
    let selected_hashes = RwSignal::new(HashSet::<String>::new());

    let sorted_hashes_data = Memo::new(move |_| {
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
        torrents
    });

    let filtered_hashes = Memo::new(move |_| {
        sorted_hashes_data.get().into_iter().map(|t| t.hash.clone()).collect::<Vec<String>>()
    });

    let selected_count = Signal::derive(move || {
        let current_hashes: HashSet<String> = filtered_hashes.get().into_iter().collect();
        selected_hashes.with(|selected| {
            selected.iter().filter(|h| current_hashes.contains(*h)).count()
        })
    });

    let handle_select_all = Callback::new(move |checked: bool| {
        selected_hashes.update(|selected| {
            let hashes = filtered_hashes.get_untracked();
            for h in hashes {
                if checked {
                    selected.insert(h);
                } else {
                    selected.remove(&h);
                }
            }
        });
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

    let on_action = Callback::new(move |(action, hash): (String, String)| {
        let (success_msg_str, error_msg_str): (&'static str, &'static str) = get_action_messages(&action);
        let success_msg = success_msg_str.to_string();
        let error_msg = error_msg_str.to_string();
        spawn_local(async move {
            let result = match action.as_str() {
                "delete" => api::torrent::delete(&hash).await,
                "delete_with_data" => api::torrent::delete_with_data(&hash).await,
                "start" => api::torrent::start(&hash).await,
                "stop" => api::torrent::stop(&hash).await,
                _ => api::torrent::action(&hash, &action).await,
            };
            match result {
                Ok(_) => show_toast(NotificationLevel::Success, success_msg),
                Err(e) => show_toast(NotificationLevel::Error, format!("{}: {:?}", error_msg, e)),
            }
        });
    });

    view! {
        <div class="h-full bg-background relative flex flex-col overflow-hidden px-4 py-2">
            // --- DESKTOP VIEW ---
            <div class="hidden md:flex flex-col h-full overflow-hidden">
                <DataTableWrapper class="flex-1 min-h-0 bg-card/50">
                    <div class="h-full overflow-auto">
                        <DataTable>
                            <DataTableHeader class="sticky top-0 bg-muted/80 backdrop-blur-sm z-10">
                                <DataTableRow class="hover:bg-transparent">
                                    <DataTableHead class="w-12 px-4">
                                        <Checkbox
                                            checked=Signal::derive(move || {
                                                let hashes = filtered_hashes.get();
                                                !hashes.is_empty() && selected_count.get() == hashes.len()
                                            })
                                            on_checked_change=handle_select_all
                                        />
                                    </DataTableHead>
                                    <DataTableHead class="cursor-pointer group select-none" on:click=move |_| handle_sort(SortColumn::Name)>
                                        <div class="flex items-center gap-2">
                                            "Name" 
                                            <ArrowUpDown class="size-3 opacity-50 group-hover:opacity-100 transition-opacity" />
                                        </div>
                                    </DataTableHead>
                                    <DataTableHead class="w-24">"Size"</DataTableHead>
                                    <DataTableHead class="w-48">"Progress"</DataTableHead>
                                    <DataTableHead class="w-24">"Status"</DataTableHead>
                                    <DataTableHead class="w-24 text-right">"DL Speed"</DataTableHead>
                                    <DataTableHead class="w-24 text-right">"UP Speed"</DataTableHead>
                                    <DataTableHead class="w-24 text-right">"ETA"</DataTableHead>
                                    <DataTableHead class="w-32 text-right">"Date"</DataTableHead>
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody>
                                <For each=move || filtered_hashes.get() key=|hash| hash.clone() children={
                                    let on_action = on_action.clone();
                                    move |hash| {
                                        let h = hash.clone();
                                        let is_selected = Signal::derive(move || {
                                            selected_hashes.with(|selected| selected.contains(&h))
                                        });
                                        let h_for_change = hash.clone();
                                        view! { 
                                            <TorrentRow 
                                                hash=hash.clone() 
                                                on_action=on_action.clone() 
                                                is_selected=is_selected
                                                on_select=Callback::new(move |checked| {
                                                    selected_hashes.update(|selected| {
                                                        if checked { selected.insert(h_for_change.clone()); }
                                                        else { selected.remove(&h_for_change); }
                                                    });
                                                })
                                            /> 
                                        }
                                    }
                                } />
                            </DataTableBody>
                        </DataTable>
                    </div>
                </DataTableWrapper>
                
                // Selection Info Footer
                <div class="flex items-center justify-between py-2 text-xs text-muted-foreground">
                    <div>
                        {move || format!("{} / {} torrent seçili", selected_count.get(), filtered_hashes.get().len())}
                    </div>
                </div>
            </div>

            // --- MOBILE VIEW ---
            <div class="md:hidden flex flex-col h-full bg-muted/10 relative overflow-hidden">
                <div class="flex-1 overflow-y-auto p-3 min-h-0">
                    <For each=move || filtered_hashes.get() key=|hash| hash.clone() children={
                        let on_action = on_action.clone();
                        move |hash| {
                            view! { 
                                 <div class="pb-3">
                                    <TorrentCard hash=hash.clone() on_action=on_action.clone() /> 
                                </div>
                            }
                        }
                    } />
                </div>
            </div>
        </div>
    }.into_any()
}

#[component]
fn TorrentRow(
    hash: String,
    on_action: Callback<(String, String)>,
    is_selected: Signal<bool>,
    on_select: Callback<bool>,
) -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let h = hash.clone();
    let torrent = Memo::new(move |_| store.torrents.with(|map| map.get(&h).cloned()));

    let stored_hash = StoredValue::new(hash.clone());

    view! {
        <Show when=move || torrent.get().is_some() fallback=|| ()>
            {
                let on_action = on_action.clone();
                move || {
                    let t = torrent.get().unwrap();
                    let t_name = t.name.clone();
                    let status_color = match t.status { shared::TorrentStatus::Seeding => "text-green-500", shared::TorrentStatus::Downloading => "text-blue-500", shared::TorrentStatus::Paused => "text-yellow-500", shared::TorrentStatus::Error => "text-red-500", _ => "text-muted-foreground" };
                    
                    let is_active_selection = Memo::new(move |_| {
                        let selected = store.selected_torrent.get();
                        selected.as_deref() == Some(stored_hash.get_value().as_str())
                    });

                    let t_name_for_title = t_name.clone();
                    let t_name_for_content = t_name.clone();
                    let h_for_menu = stored_hash.get_value();

                    view! {
                        <TorrentContextMenu torrent_hash=h_for_menu on_action=on_action.clone()>
                            <DataTableRow 
                                class="cursor-pointer group h-10"
                                attr:data-state=move || if is_selected.get() || is_active_selection.get() { "selected" } else { "" }
                                on:click=move |_| store.selected_torrent.set(Some(stored_hash.get_value()))
                            >
                                <DataTableCell class="w-12 px-4">
                                    <Checkbox 
                                        checked=is_selected 
                                        on_checked_change=on_select 
                                    />
                                </DataTableCell>
                                <DataTableCell class="font-medium truncate max-w-[200px] lg:max-w-md" attr:title=t_name_for_title>
                                    {t_name_for_content}
                                </DataTableCell>
                                <DataTableCell class="font-mono text-xs text-muted-foreground">
                                    {format_bytes(t.size)}
                                </DataTableCell>
                                <DataTableCell>
                                    <div class="flex items-center gap-2">
                                        <div class="h-1.5 w-full bg-secondary rounded-full overflow-hidden">
                                            <div class="h-full bg-primary transition-all duration-500" style=format!("width: {}%", t.percent_complete)></div>
                                        </div>
                                        <span class="text-[10px] text-muted-foreground w-10 text-right">{format!("{:.1}%", t.percent_complete)}</span>
                                    </div>
                                </DataTableCell>
                                <DataTableCell class={format!("text-xs font-semibold {}", status_color)}>
                                    {format!("{:?}", t.status)}
                                </DataTableCell>
                                <DataTableCell class="text-right font-mono text-xs text-green-600 dark:text-green-500 whitespace-nowrap">
                                    {format_speed(t.down_rate)}
                                </DataTableCell>
                                <DataTableCell class="text-right font-mono text-xs text-blue-600 dark:text-blue-500 whitespace-nowrap">
                                    {format_speed(t.up_rate)}
                                </DataTableCell>
                                <DataTableCell class="text-right font-mono text-xs text-muted-foreground whitespace-nowrap">
                                    {format_duration(t.eta)}
                                </DataTableCell>
                                <DataTableCell class="text-right font-mono text-xs text-muted-foreground whitespace-nowrap">
                                    {format_date(t.added_date)}
                                </DataTableCell>
                            </DataTableRow>
                        </TorrentContextMenu>
                    }.into_any()
                }
            }
        </Show>
    }.into_any()
}

#[component]
fn TorrentCard(
    hash: String,
    on_action: Callback<(String, String)>,
) -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let h = hash.clone();
    let torrent = Memo::new(move |_| store.torrents.with(|map| map.get(&h).cloned()));

    let stored_hash = StoredValue::new(hash.clone());

    view! {
        <Show when=move || torrent.get().is_some() fallback=|| ()>
            {
                let on_action = on_action.clone();
                move || {
                    let t = torrent.get().unwrap();
                    let t_name = t.name.clone();
                    let status_badge_class = match t.status { shared::TorrentStatus::Seeding => "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400 border-green-200 dark:border-green-800", shared::TorrentStatus::Downloading => "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400 border-blue-200 dark:border-blue-800", shared::TorrentStatus::Paused => "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400 border-yellow-200 dark:border-yellow-800", shared::TorrentStatus::Error => "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400 border-red-200 dark:border-red-800", _ => "bg-muted text-muted-foreground" };
                    let h_for_menu = stored_hash.get_value();

                    view! {
                        <TorrentContextMenu torrent_hash=h_for_menu on_action=on_action.clone()>
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
                                <CardBody class="p-3 pt-2 gap-3 flex flex-col">
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
                                </CardBody>
                            </Card>
                            </div>
                        </TorrentContextMenu>
                    }.into_any()
                }
            }
        </Show>
    }.into_any()
}
