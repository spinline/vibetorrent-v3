use leptos::*;




fn format_bytes(bytes: i64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    let i = (bytes as f64).log2().div_euclid(10.0) as usize;
    format!("{:.1} {}", (bytes as f64) / 1024_f64.powi(i as i32), UNITS[i])
}

fn format_speed(bytes_per_sec: i64) -> String {
    if bytes_per_sec == 0 {
        return "0 B/s".to_string();
    }
    format!("{}/s", format_bytes(bytes_per_sec))
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortDirection {
    Ascending,
    Descending,
}

#[component]
pub fn TorrentTable() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    
    let sort_col = create_rw_signal(SortColumn::Name);
    let sort_dir = create_rw_signal(SortDirection::Ascending);

    let filtered_torrents = move || {
        let mut torrents = store.torrents.get().into_iter().filter(|t| {
            let filter = store.filter.get();
            match filter {
                crate::store::FilterStatus::All => true,
                crate::store::FilterStatus::Downloading => t.status == shared::TorrentStatus::Downloading,
                crate::store::FilterStatus::Seeding => t.status == shared::TorrentStatus::Seeding,
                crate::store::FilterStatus::Completed => t.status == shared::TorrentStatus::Seeding || t.status == shared::TorrentStatus::Paused, // Approximate
                crate::store::FilterStatus::Inactive => t.status == shared::TorrentStatus::Paused || t.status == shared::TorrentStatus::Error,
                 _ => true
            }
        }).collect::<Vec<_>>();

        torrents.sort_by(|a, b| {
            let col = sort_col.get();
            let dir = sort_dir.get();
            let cmp = match col {
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Size => a.size.cmp(&b.size),
                SortColumn::Progress => a.percent_complete.partial_cmp(&b.percent_complete).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Status => format!("{:?}", a.status).cmp(&format!("{:?}", b.status)),
                SortColumn::DownSpeed => a.down_rate.cmp(&b.down_rate),
                SortColumn::UpSpeed => a.up_rate.cmp(&b.up_rate),
                SortColumn::ETA => {
                     // ETA 0 means infinity usually, so we need to handle it.
                     // But for simple sorting, maybe just treat is numeric?
                     // Let's treat 0 as MAX for ascending, MIN for descending? Or just as is?
                     // Usually negative or 0 means unknown/inf.
                     // Let's handle 0 as very large number for sorting purposes if we want it at the end of ascending
                     let a_eta = if a.eta <= 0 { i64::MAX } else { a.eta };
                     let b_eta = if b.eta <= 0 { i64::MAX } else { b.eta };
                     a_eta.cmp(&b_eta)
                }
            };
            if dir == SortDirection::Descending { cmp.reverse() } else { cmp }
        });
        
        torrents
    };

    let handle_sort = move |col: SortColumn| {
        if sort_col.get() == col {
            sort_dir.update(|d| *d = match d {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            });
        } else {
            sort_col.set(col);
            sort_dir.set(SortDirection::Ascending);
        }
    };

    let sort_arrow = move |col: SortColumn| {
        if sort_col.get() == col {
            match sort_dir.get() {
                SortDirection::Ascending => view!{ <span class="ml-1 text-xs">"▲"</span> }.into_view(),
                SortDirection::Descending => view!{ <span class="ml-1 text-xs">"▼"</span> }.into_view(),
            }
        } else {
            view!{ <span class="ml-1 text-xs opacity-0 group-hover:opacity-50">"▲"</span> }.into_view()
        }
    };

    view! {
        <div class="overflow-x-auto h-full bg-base-100">
            <table class="table table-xs table-pin-rows w-full max-w-full">
                <thead>
                    <tr class="bg-base-200 text-base-content/70">
                        <th class="w-8">
                            <label>
                                <input type="checkbox" class="checkbox checkbox-xs rounded-none" />
                            </label>
                        </th>
                        <th class="cursor-pointer hover:bg-base-300 transition-colors group select-none" on:click=move |_| handle_sort(SortColumn::Name)>
                            <div class="flex items-center">"Name" {move || sort_arrow(SortColumn::Name)}</div>
                        </th>
                        <th class="w-24 cursor-pointer hover:bg-base-300 transition-colors group select-none" on:click=move |_| handle_sort(SortColumn::Size)>
                             <div class="flex items-center">"Size" {move || sort_arrow(SortColumn::Size)}</div>
                        </th>
                        <th class="w-48 cursor-pointer hover:bg-base-300 transition-colors group select-none" on:click=move |_| handle_sort(SortColumn::Progress)>
                             <div class="flex items-center">"Progress" {move || sort_arrow(SortColumn::Progress)}</div>
                        </th>
                         <th class="w-24 cursor-pointer hover:bg-base-300 transition-colors group select-none" on:click=move |_| handle_sort(SortColumn::Status)>
                             <div class="flex items-center">"Status" {move || sort_arrow(SortColumn::Status)}</div>
                        </th>
                        // <th class="w-20">"Seeds"</th> // Not available in shared::Torrent
                        // <th class="w-20">"Peers"</th> // Not available in shared::Torrent
                        <th class="w-24 cursor-pointer hover:bg-base-300 transition-colors group select-none" on:click=move |_| handle_sort(SortColumn::DownSpeed)>
                             <div class="flex items-center">"Down Speed" {move || sort_arrow(SortColumn::DownSpeed)}</div>
                        </th>
                        <th class="w-24 cursor-pointer hover:bg-base-300 transition-colors group select-none" on:click=move |_| handle_sort(SortColumn::UpSpeed)>
                             <div class="flex items-center">"Up Speed" {move || sort_arrow(SortColumn::UpSpeed)}</div>
                        </th>
                         <th class="w-24 cursor-pointer hover:bg-base-300 transition-colors group select-none" on:click=move |_| handle_sort(SortColumn::ETA)>
                             <div class="flex items-center">"ETA" {move || sort_arrow(SortColumn::ETA)}</div>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {move || filtered_torrents().into_iter().map(|t| {
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
                            <tr class="hover group border-b border-base-200">
                                <th>
                                    <label>
                                        <input type="checkbox" class="checkbox checkbox-xs rounded-none" />
                                    </label>
                                </th>
                                <td class="font-medium truncate max-w-xs" title={t.name.clone()}>
                                    {t.name}
                                </td>
                                <td class="opacity-80 font-mono text-[11px]">{format_bytes(t.size)}</td>
                                <td>
                                    <div class="flex items-center gap-2">
                                        <progress class={format!("progress w-24 {}", progress_class)} value={t.percent_complete} max="100"></progress>
                                        <span class="text-[10px] opacity-70">{format!("{:.1}%", t.percent_complete)}</span>
                                    </div>
                                </td>
                                <td class={format!("text-[11px] font-medium {}", status_class)}>{status_str}</td>
                                // <td class="text-right font-mono text-[11px] opacity-80">-</td>
                                // <td class="text-right font-mono text-[11px] opacity-80">-</td>
                                <td class="text-right font-mono text-[11px] opacity-80 text-success">{format_speed(t.down_rate)}</td>
                                <td class="text-right font-mono text-[11px] opacity-80 text-primary">{format_speed(t.up_rate)}</td>
                                <td class="text-right font-mono text-[11px] opacity-80">{if t.eta > 0 { format!("{}s", t.eta) } else { "∞".to_string() }}</td> // Temporary ETA format
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}
