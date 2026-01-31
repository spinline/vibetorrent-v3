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

#[component]
pub fn TorrentTable() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    let filtered_torrents = move || {
        store.torrents.get().into_iter().filter(|t| {
            let filter = store.filter.get();
            match filter {
                crate::store::FilterStatus::All => true,
                crate::store::FilterStatus::Downloading => t.status == shared::TorrentStatus::Downloading,
                crate::store::FilterStatus::Seeding => t.status == shared::TorrentStatus::Seeding,
                crate::store::FilterStatus::Completed => t.status == shared::TorrentStatus::Seeding || t.status == shared::TorrentStatus::Paused, // Approximate
                crate::store::FilterStatus::Inactive => t.status == shared::TorrentStatus::Paused || t.status == shared::TorrentStatus::Error,
                 _ => true
            }
        }).collect::<Vec<_>>()
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
                        <th>"Name"</th>
                        <th class="w-24">"Size"</th>
                        <th class="w-48">"Progress"</th>
                        <th class="w-24">"Status"</th>
                        // <th class="w-20">"Seeds"</th> // Not available in shared::Torrent
                        // <th class="w-20">"Peers"</th> // Not available in shared::Torrent
                        <th class="w-24">"Down Speed"</th>
                        <th class="w-24">"Up Speed"</th>
                        <th class="w-24">"ETA"</th>
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
