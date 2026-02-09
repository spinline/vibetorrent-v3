use leptos::prelude::*;
use leptos::logging;
use leptos::html;
use leptos::task::spawn_local;
use leptos_use::{on_click_outside, use_timeout_fn};
use crate::store::{get_action_messages, show_toast_with_signal, FilterStatus};
use crate::api;
use shared::{NotificationLevel, Torrent};
use std::collections::HashMap;

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

fn format_duration(seconds: i64) -> String {
    if seconds <= 0 {
        return "∞".to_string();
    }

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

fn format_date(timestamp: i64) -> String {
    if timestamp <= 0 {
        return "N/A".to_string();
    }
    let dt = chrono::DateTime::from_timestamp(timestamp, 0);
    match dt {
        Some(dt) => dt.format("%d/%m/%Y %H:%M").to_string(),
        None => "N/A".to_string(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortColumn {
    Name,
    Size,
    Progress,
    Status,
    DownSpeed,
    UpSpeed,
    ETA,
    AddedDate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortDirection {
    Ascending,
    Descending,
}

#[component]
pub fn TorrentTable() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    let sort_col = create_rw_signal(SortColumn::AddedDate);
    let sort_dir = create_rw_signal(SortDirection::Descending);

    // Get sorted and filtered hashes only
    let filtered_hashes = move || {
        store.torrents.with(|map| {
            let mut torrents: Vec<&shared::Torrent> = map
                .values()
                .filter(|t| {
                    let filter = store.filter.get();
                    let search = store.search_query.get().to_lowercase();

                    let matches_filter = match filter {
                        crate::store::FilterStatus::All => true,
                        crate::store::FilterStatus::Downloading => {
                            t.status == shared::TorrentStatus::Downloading
                        }
                        crate::store::FilterStatus::Seeding => {
                            t.status == shared::TorrentStatus::Seeding
                        }
                        crate::store::FilterStatus::Completed => {
                            t.status == shared::TorrentStatus::Seeding
                                || (t.status == shared::TorrentStatus::Paused
                                    && t.percent_complete >= 100.0)
                        }
                        crate::store::FilterStatus::Paused => {
                            t.status == shared::TorrentStatus::Paused
                        }
                        crate::store::FilterStatus::Inactive => {
                            t.status == shared::TorrentStatus::Paused
                                || t.status == shared::TorrentStatus::Error
                        }
                        _ => true,
                    };

                    let matches_search = if search.is_empty() {
                        true
                    } else {
                        t.name.to_lowercase().contains(&search)
                    };

                    matches_filter && matches_search
                })
                .collect();

            torrents.sort_by(|a, b| {
                let col = sort_col.get();
                let dir = sort_dir.get();
                let cmp = match col {
                    SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortColumn::Size => a.size.cmp(&b.size),
                    SortColumn::Progress => a
                        .percent_complete
                        .partial_cmp(&b.percent_complete)
                        .unwrap_or(std::cmp::Ordering::Equal),
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
                if dir == SortDirection::Descending {
                    cmp.reverse()
                } else {
                    cmp
                }
            });

            torrents.into_iter().map(|t| t.hash.clone()).collect::<Vec<String>>()
        })
    };

    let handle_sort = move |col: SortColumn| {
        if sort_col.get() == col {
            sort_dir.update(|d| {
                *d = match d {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                }
            });
        } else {
            sort_col.set(col);
            sort_dir.set(SortDirection::Ascending);
        }
    };

    // Refs for click outside detection
    let sort_details_ref = create_node_ref::<html::Details>();
    let _ = on_click_outside(sort_details_ref, move |_| {
        if let Some(el) = sort_details_ref.get_untracked() {
            el.set_open(false);
        }
    });

    let sort_arrow = move |col: SortColumn| {
        if sort_col.get() == col {
            match sort_dir.get() {
                SortDirection::Ascending => {
                    view! { <span class="ml-1 text-xs">"▲"</span> }.into_any()
                }
                SortDirection::Descending => {
                    view! { <span class="ml-1 text-xs">"▼"</span> }.into_any()
                }
            }
        } else {
            view! { <span class="ml-1 text-xs opacity-0 group-hover:opacity-50">"▲"</span> }
                .into_any()
        }
    };

    let (selected_hash, set_selected_hash) = create_signal(Option::<String>::None);
    let (menu_visible, set_menu_visible) = create_signal(false);
    let (menu_position, set_menu_position) = create_signal((0, 0));

    let handle_context_menu = move |e: web_sys::MouseEvent, hash: String| {
        e.prevent_default();
        set_menu_position.set((e.client_x(), e.client_y()));
        set_selected_hash.set(Some(hash)); // Select on right click too
        set_menu_visible.set(true);
    };

    let on_action = move |(action, hash): (String, String)| {
        logging::log!("TorrentTable Action: {} on {}", action, hash);

        let (success_msg, error_msg) = get_action_messages(&action);
        let success_msg = success_msg.to_string();
        let error_msg = error_msg.to_string();

        let notifications = store.notifications;

        let hash = hash.clone();
        let action = action.clone();

        spawn_local(async move {
            let result = match action.as_str() {
                "delete" => api::torrent::delete(&hash).await,
                "delete_with_data" => api::torrent::delete_with_data(&hash).await,
                "start" => api::torrent::start(&hash).await,
                "stop" => api::torrent::stop(&hash).await,
                _ => api::torrent::action(&hash, &action).await,
            };

            match result {
                Ok(_) => {
                    logging::log!("Action {} executed successfully", action);
                    show_toast_with_signal(notifications, NotificationLevel::Success, success_msg);
                }
                Err(e) => {
                    logging::error!("Action failed: {:?}", e);
                    show_toast_with_signal(notifications, NotificationLevel::Error, format!("{}: {:?}", error_msg, e));
                }
            }
        });
    };

    view! {
        <div class="overflow-x-auto h-full bg-base-100 relative">
            <div class="hidden md:block h-full overflow-x-auto">
                <table class="table table-sm table-pin-rows w-full max-w-full whitespace-nowrap">
                    <thead>
                        <tr class="text-xs uppercase text-base-content/60 border-b border-base-200">
                            <th class="cursor-pointer hover:bg-base-300 group select-none" on:click=move |_| handle_sort(SortColumn::Name)>
                                <div class="flex items-center">"Name" {move || sort_arrow(SortColumn::Name)}</div>
                            </th>
                            <th class="w-24 cursor-pointer hover:bg-base-300 group select-none" on:click=move |_| handle_sort(SortColumn::Size)>
                                 <div class="flex items-center">"Size" {move || sort_arrow(SortColumn::Size)}</div>
                            </th>
                            <th class="w-48 cursor-pointer hover:bg-base-300 group select-none" on:click=move |_| handle_sort(SortColumn::Progress)>
                                 <div class="flex items-center">"Progress" {move || sort_arrow(SortColumn::Progress)}</div>
                            </th>
                            <th class="w-24 cursor-pointer hover:bg-base-300 group select-none" on:click=move |_| handle_sort(SortColumn::Status)>
                                 <div class="flex items-center">"Status" {move || sort_arrow(SortColumn::Status)}</div>
                            </th>
                            <th class="w-24 cursor-pointer hover:bg-base-300 group select-none" on:click=move |_| handle_sort(SortColumn::DownSpeed)>
                                 <div class="flex items-center">"DL Speed" {move || sort_arrow(SortColumn::DownSpeed)}</div>
                            </th>
                            <th class="w-24 cursor-pointer hover:bg-base-300 group select-none" on:click=move |_| handle_sort(SortColumn::UpSpeed)>
                                 <div class="flex items-center">"Up Speed" {move || sort_arrow(SortColumn::UpSpeed)}</div>
                            </th>
                            <th class="w-24 cursor-pointer hover:bg-base-300 group select-none" on:click=move |_| handle_sort(SortColumn::ETA)>
                                 <div class="flex items-center">"ETA" {move || sort_arrow(SortColumn::ETA)}</div>
                            </th>
                            <th class="w-32 cursor-pointer hover:bg-base-300 group select-none" on:click=move |_| handle_sort(SortColumn::AddedDate)>
                                 <div class="flex items-center">"Date" {move || sort_arrow(SortColumn::AddedDate)}</div>
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || filtered_hashes()
                            key=|hash| hash.clone()
                            children={
                                let handle_context_menu = handle_context_menu.clone();
                                move |hash| {
                                    view! {
                                        <TorrentRow
                                            hash=hash.clone()
                                            selected_hash=selected_hash
                                            set_selected_hash=set_selected_hash
                                            on_context_menu=handle_context_menu.clone()
                                        />
                                    }
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>

            <div class="md:hidden flex flex-col h-full bg-base-200 relative cursor-pointer">
                <div class="px-3 py-2 border-b border-base-200 flex justify-between items-center bg-base-100/95 backdrop-blur z-10 shrink-0 cursor-default">
                    <span class="text-xs font-bold opacity-50 uppercase tracking-wider">"Torrents"</span>

                    <details class="dropdown dropdown-end" node_ref=sort_details_ref>
                        <summary class="btn btn-ghost btn-xs gap-1 opacity-70 font-normal list-none [&::-webkit-details-marker]:hidden cursor-pointer">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4 pointer-events-none">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M3 7.5L7.5 3m0 0L12 7.5M7.5 3v13.5m13.5 0L16.5 21m0 0L12 16.5m4.5 4.5V7.5" />
                            </svg>
                            <span class="pointer-events-none">"Sort"</span>
                        </summary>
                        <ul class="dropdown-content z-[100] menu p-2 shadow bg-base-100 rounded-box w-48 mt-1 border border-base-200 text-xs cursor-default">
                                <li class="menu-title px-2 py-1 opacity-50 text-[10px] uppercase font-bold">"Sort By"</li>
                                {
                                    let columns = vec![
                                        (SortColumn::Name, "Name"),
                                        (SortColumn::Size, "Size"),
                                        (SortColumn::Progress, "Progress"),
                                        (SortColumn::Status, "Status"),
                                        (SortColumn::DownSpeed, "DL Speed"),
                                        (SortColumn::UpSpeed, "Up Speed"),
                                        (SortColumn::ETA, "ETA"),
                                        (SortColumn::AddedDate, "Date"),
                                    ];

                                    columns.into_iter().map(|(col, label)| {
                                        let is_active = move || sort_col.get() == col;
                                        let current_dir = move || sort_dir.get();

                                        view! {
                                            <li>
                                                <button
                                                    type="button"
                                                    class=move || if is_active() { "bg-primary/10 text-primary font-bold flex justify-between" } else { "flex justify-between" }
                                                    on:click=move |_| {
                                                        handle_sort(col);
                                                        if let Some(el) = sort_details_ref.get_untracked() {
                                                            el.set_open(false);
                                                        }
                                                    }
                                                >
                                                    {label}
                                                    <Show when=is_active fallback=|| ()>
                                                        <span class="opacity-70 text-[10px]">
                                                            {move || match current_dir() {
                                                                SortDirection::Ascending => "▲",
                                                                SortDirection::Descending => "▼",
                                                            }}
                                                        </span>
                                                    </Show>
                                                </button>
                                            </li>
                                        }
                                    }).collect::<Vec<_>>()
                                }
                        </ul>
                    </details>
                </div>

                <div class="overflow-y-auto p-3 pb-20 flex-1 grid grid-cols-1 content-start gap-3 cursor-pointer">
                    <For
                        each=move || filtered_hashes()
                        key=|hash| hash.clone()
                        children={
                            let handle_context_menu = handle_context_menu.clone();
                            move |hash| {
                                view! {
                                    <TorrentCard
                                        hash=hash.clone()
                                        selected_hash=selected_hash
                                        set_selected_hash=set_selected_hash
                                        set_menu_position=set_menu_position
                                        set_menu_visible=set_menu_visible
                                        on_context_menu=handle_context_menu.clone()
                                    />
                                }
                            }
                        }
                    />
                </div>
            </div>

            <Show when=move || menu_visible.get() fallback=|| ()>
                <crate::components::context_menu::ContextMenu
                    visible=true
                    position=menu_position.get()
                    torrent_hash=selected_hash.get().unwrap_or_default()
                    on_close=Callback::new(move |()| set_menu_visible.set(false))
                    on_action=Callback::new(move |args| on_action(args))
                />
            </Show>
        </div>
    }
}

#[component]
fn TorrentRow(
    hash: String,
    selected_hash: ReadSignal<Option<String>>,
    set_selected_hash: WriteSignal<Option<String>>,
    on_context_menu: impl Fn(web_sys::MouseEvent, String) + 'static + Clone + Send + Sync,
) -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    let h = hash.clone();
    // Memoized access to the specific torrent data.
    // This only re-renders the row if this specific torrent actually changes.
    let torrent = create_memo(move |_| {
        store.torrents.with(|map| map.get(&h).cloned())
    });

    view! {
        <Show when=move || torrent.get().is_some() fallback=|| ()>
            {
                let on_context_menu = on_context_menu.clone();
                let hash = hash.clone();

                move || {
                    let t = torrent.get().unwrap();
                    let t_hash = hash.clone();
                    let t_hash_class = t_hash.clone();
                    let on_context_menu = on_context_menu.clone();

                    let progress_class = if t.percent_complete >= 100.0 { "progress-success" } else { "progress-primary" };
                    let status_str = format!("{:?}", t.status);
                    let status_class = match t.status {
                        shared::TorrentStatus::Seeding => "text-success",
                        shared::TorrentStatus::Downloading => "text-primary",
                        shared::TorrentStatus::Paused => "text-warning",
                        shared::TorrentStatus::Error => "text-error",
                        _ => "text-base-content/50"
                    };

                    view! {
                        <tr
                            class=move || {
                                let base = "hover border-b border-base-200 select-none";
                                if selected_hash.get() == Some(t_hash_class.clone()) {
                                    format!("{} bg-primary/10", base)
                                } else {
                                    base.to_string()
                                }
                            }
                            on:contextmenu={
                                let t_hash = t_hash.clone();
                                let on_context_menu = on_context_menu.clone();
                                move |e: web_sys::MouseEvent| on_context_menu(e, t_hash.clone())
                            }
                            on:click={
                                let t_hash = t_hash.clone();
                                move |_| set_selected_hash.set(Some(t_hash.clone()))
                            }
                        >
                            <td class="font-medium truncate max-w-xs" title={t.name.clone()}>
                                {t.name.clone()}
                            </td>
                            <td class="opacity-80 font-mono text-[11px]">{format_bytes(t.size)}</td>
                            <td>
                                <div class="flex items-center gap-2">
                                    <progress class={format!("progress w-24 {}", progress_class)} value={t.percent_complete} max="100"></progress>
                                    <span class="text-[10px] opacity-70">{format!("{:.1}%", t.percent_complete)}</span>
                                </div>
                            </td>
                            <td class={format!("text-[11px] font-medium {}", status_class)}>{status_str}</td>
                            <td class="text-right font-mono text-[11px] opacity-80 text-success">{format_speed(t.down_rate)}</td>
                            <td class="text-right font-mono text-[11px] opacity-80 text-primary">{format_speed(t.up_rate)}</td>
                            <td class="text-right font-mono text-[11px] opacity-80">{format_duration(t.eta)}</td>
                            <td class="text-right font-mono text-[11px] opacity-80 whitespace-nowrap">{format_date(t.added_date)}</td>
                        </tr>
                    }
                }
            }
        </Show>
    }
}

#[component]
fn TorrentCard(
    hash: String,
    selected_hash: ReadSignal<Option<String>>,
    set_selected_hash: WriteSignal<Option<String>>,
    set_menu_position: WriteSignal<(i32, i32)>,
    set_menu_visible: WriteSignal<bool>,
    on_context_menu: impl Fn(web_sys::MouseEvent, String) + 'static + Clone + Send + Sync,
) -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    let h = hash.clone();
    let torrent = create_memo(move |_| {
        store.torrents.with(|map| map.get(&h).cloned())
    });

    view! {
        <Show when=move || torrent.get().is_some() fallback=|| ()>
            {
                let hash = hash.clone();
                let on_context_menu = on_context_menu.clone();

                move || {
                    let t = torrent.get().unwrap();
                    let t_hash = hash.clone();
                    let t_hash_class = t_hash.clone();
                    let on_context_menu = on_context_menu.clone();

                    let progress_class = if t.percent_complete >= 100.0 { "progress-success" } else { "progress-primary" };
                    let status_str = format!("{:?}", t.status);
                    let status_badge_class = match t.status {
                        shared::TorrentStatus::Seeding => "badge-success badge-soft",
                        shared::TorrentStatus::Downloading => "badge-primary badge-soft",
                        shared::TorrentStatus::Paused => "badge-warning badge-soft",
                        shared::TorrentStatus::Error => "badge-error badge-soft",
                        _ => "badge-ghost"
                    };

                    let t_hash_long = t_hash.clone();
                    let leptos_use::UseTimeoutFnReturn { start, stop, .. } = use_timeout_fn(
                        move |pos: (i32, i32)| {
                            set_menu_position.set(pos);
                            set_selected_hash.set(Some(t_hash_long.clone()));
                            set_menu_visible.set(true);

                            // Haptic feedback
                            let navigator = window().navigator();
                            if let Ok(vibrate) = js_sys::Reflect::get(&navigator, &"vibrate".into()) {
                                if vibrate.is_function() {
                                    let _ = navigator.vibrate_with_duration(50);
                                }
                            }
                        },
                        600.0,
                    );

                    let handle_touchstart = {
                        let start = start.clone();
                        move |e: web_sys::TouchEvent| {
                            if let Some(touch) = e.touches().get(0) {
                                start((touch.client_x(), touch.client_y()));
                            }
                        }
                    };

                    let handle_touchmove = {
                        let stop = stop.clone();
                        move |_| stop()
                    };

                    let handle_touchend = {
                        let stop = stop.clone();
                        move |_| stop()
                    };

                    let handle_touchcancel = move |_| stop();

                    view! {
                        <div
                            class=move || {
                                let base = "card card-compact bg-base-100 shadow-sm border border-base-200 transition-transform active:scale-[0.99] select-none cursor-pointer";
                                if selected_hash.get() == Some(t_hash_class.clone()) {
                                    format!("{} ring-2 ring-primary ring-inset", base)
                                } else {
                                    base.to_string()
                                }
                            }
                            style="user-select: none; -webkit-user-select: none; -webkit-touch-callout: none;"
                            on:contextmenu={
                                let t_hash = t_hash.clone();
                                let on_context_menu = on_context_menu.clone();
                                move |e: web_sys::MouseEvent| on_context_menu(e, t_hash.clone())
                            }
                            on:click={
                                let t_hash = t_hash.clone();
                                move |_| set_selected_hash.set(Some(t_hash.clone()))
                            }
                            on:touchstart=handle_touchstart
                            on:touchmove=handle_touchmove
                            on:touchend=handle_touchend
                            on:touchcancel=handle_touchcancel
                        >
                            <div class="card-body gap-3">
                                <div class="flex justify-between items-start gap-2">
                                    <h3 class="font-medium text-sm line-clamp-2 leading-tight">{t.name}</h3>
                                    <div class={format!("badge badge-xs text-[10px] whitespace-nowrap {}", status_badge_class)}>
                                        {status_str}
                                    </div>
                                </div>

                                <div class="flex flex-col gap-1">
                                    <div class="flex justify-between text-[10px] opacity-70">
                                        <span>{format_bytes(t.size)}</span>
                                        <span>{format!("{:.1}%", t.percent_complete)}</span>
                                    </div>
                                    <progress class={format!("progress w-full h-1.5 {}", progress_class)} value={t.percent_complete} max="100"></progress>
                                </div>

                                <div class="grid grid-cols-4 gap-2 text-[10px] font-mono opacity-80 pt-1 border-t border-base-200/50">
                                    <div class="flex flex-col">
                                        <span class="text-[9px] opacity-60 uppercase">"Down"</span>
                                        <span class="text-success">{format_speed(t.down_rate)}</span>
                                    </div>
                                    <div class="flex flex-col text-center border-l border-r border-base-200/50">
                                        <span class="text-[9px] opacity-60 uppercase">"Up"</span>
                                        <span class="text-primary">{format_speed(t.up_rate)}</span>
                                    </div>
                                    <div class="flex flex-col text-center border-r border-base-200/50">
                                        <span class="text-[9px] opacity-60 uppercase">"ETA"</span>
                                        <span>{format_duration(t.eta)}</span>
                                    </div>
                                    <div class="flex flex-col text-right">
                                        <span class="text-[9px] opacity-60 uppercase">"Date"</span>
                                        <span>{format_date(t.added_date)}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                }
            }
        </Show>
    }
}