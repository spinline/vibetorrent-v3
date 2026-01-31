use leptos::*;

#[derive(Clone)]
struct Torrent {
    id: u32,
    name: String,
    size: String,
    progress: f32,
    status: String,
    seeds: u32,
    peers: u32,
    down_speed: String,
    up_speed: String,
}

#[component]
pub fn TorrentTable() -> impl IntoView {
    let torrents = vec![
        Torrent {
            id: 1,
            name: "Ubuntu 22.04.3 LTS".to_string(),
            size: "4.7 GB".to_string(),
            progress: 100.0,
            status: "Seeding".to_string(),
            seeds: 452,
            peers: 12,
            down_speed: "0 KB/s".to_string(),
            up_speed: "1.2 MB/s".to_string(),
        },
        Torrent {
            id: 2,
            name: "Debian 12.1.0 DVD".to_string(),
            size: "3.9 GB".to_string(),
            progress: 45.5,
            status: "Downloading".to_string(),
            seeds: 120,
            peers: 45,
            down_speed: "4.5 MB/s".to_string(),
            up_speed: "50 KB/s".to_string(),
        },
        Torrent {
            id: 3,
            name: "Arch Linux 2023.09.01".to_string(),
            size: "800 MB".to_string(),
            progress: 12.0,
            status: "Downloading".to_string(),
            seeds: 85,
            peers: 20,
            down_speed: "2.1 MB/s".to_string(),
            up_speed: "10 KB/s".to_string(),
        },
         Torrent {
            id: 4,
            name: "Fedora Workstation 39".to_string(),
            size: "2.1 GB".to_string(),
            progress: 0.0,
            status: "Paused".to_string(),
            seeds: 0,
            peers: 0,
            down_speed: "0 KB/s".to_string(),
            up_speed: "0 KB/s".to_string(),
        },
    ];

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
                        <th class="w-20">"Seeds"</th>
                        <th class="w-20">"Peers"</th>
                        <th class="w-24">"Down Speed"</th>
                        <th class="w-24">"Up Speed"</th>
                    </tr>
                </thead>
                <tbody>
                    {torrents.into_iter().map(|t| {
                        let progress_class = if t.progress == 100.0 { "progress-success" } else { "progress-primary" };
                        let status_class = match t.status.as_str() {
                            "Seeding" => "text-success",
                            "Downloading" => "text-primary",
                            "Paused" => "text-warning",
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
                                <td class="opacity-80 font-mono text-[11px]">{t.size}</td>
                                <td>
                                    <div class="flex items-center gap-2">
                                        <progress class={format!("progress w-24 {}", progress_class)} value={t.progress} max="100"></progress>
                                        <span class="text-[10px] opacity-70">{format!("{:.1}%", t.progress)}</span>
                                    </div>
                                </td>
                                <td class={format!("text-[11px] font-medium {}", status_class)}>{t.status}</td>
                                <td class="text-right font-mono text-[11px] opacity-80">{t.seeds}</td>
                                <td class="text-right font-mono text-[11px] opacity-80">{t.peers}</td>
                                <td class="text-right font-mono text-[11px] opacity-80 text-success">{t.down_speed}</td>
                                <td class="text-right font-mono text-[11px] opacity-80 text-primary">{t.up_speed}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}
