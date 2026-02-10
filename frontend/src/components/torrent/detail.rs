use leptos::prelude::*;
use leptos_shadcn_tabs::{Tabs, TabsList, TabsTrigger, TabsContent};

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

fn format_date(timestamp: i64) -> String {
    if timestamp <= 0 { return "N/A".to_string(); }
    let dt = chrono::DateTime::from_timestamp(timestamp, 0);
    match dt { Some(dt) => dt.format("%d/%m/%Y %H:%M").to_string(), None => "N/A".to_string() }
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

#[component]
pub fn TorrentDetail() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    let torrent = Memo::new(move |_| {
        let hash = store.selected_torrent.get()?;
        store.torrents.with(|map| map.get(&hash).cloned())
    });

    let close = move |_| {
        store.selected_torrent.set(None);
    };

    view! {
        <Show when=move || torrent.get().is_some()>
            {move || {
                let t = torrent.get().unwrap();
                let name = t.name.clone();
                let status_color = match t.status {
                    shared::TorrentStatus::Seeding => "text-green-500",
                    shared::TorrentStatus::Downloading => "text-blue-500",
                    shared::TorrentStatus::Paused => "text-yellow-500",
                    shared::TorrentStatus::Error => "text-red-500",
                    _ => "text-muted-foreground",
                };
                let status_text = format!("{:?}", t.status);

                view! {
                    <div class="border-t border-border bg-card flex flex-col" style="height: 280px; min-height: 200px;">
                        // Header
                        <div class="flex items-center justify-between px-4 py-2 border-b border-border bg-muted/30">
                            <div class="flex items-center gap-3 min-w-0 flex-1">
                                <h3 class="text-sm font-semibold truncate">{name}</h3>
                                <span class={format!("text-xs font-medium {}", status_color)}>{status_text}</span>
                            </div>
                            <button
                                class="inline-flex items-center justify-center rounded-md text-sm font-medium hover:bg-accent hover:text-accent-foreground h-7 w-7 text-muted-foreground shrink-0"
                                on:click=close
                                title="Close"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>

                        // Tabs
                        <Tabs default_value="general" class="flex-1 flex flex-col overflow-hidden">
                            <div class="px-4 pt-2">
                                <TabsList class="w-full">
                                    <TabsTrigger value="general">"General"</TabsTrigger>
                                    <TabsTrigger value="transfer">"Transfer"</TabsTrigger>
                                    <TabsTrigger value="files">"Files"</TabsTrigger>
                                    <TabsTrigger value="peers">"Peers"</TabsTrigger>
                                </TabsList>
                            </div>

                            <TabsContent value="general" class="flex-1 overflow-y-auto px-4 pb-3">
                                <div class="grid grid-cols-2 md:grid-cols-4 gap-x-6 gap-y-2 text-sm">
                                    <DetailItem label="Size" value=format_bytes(t.size) />
                                    <DetailItem label="Downloaded" value=format_bytes(t.completed) />
                                    <DetailItem label="Progress" value=format!("{:.1}%", t.percent_complete) />
                                    <DetailItem label="Added" value=format_date(t.added_date) />
                                    <DetailItem label="Hash" value={
                                        let hash = store.selected_torrent.get().unwrap_or_default();
                                        format!("{}…", &hash[..std::cmp::min(16, hash.len())])
                                    } />
                                    <DetailItem label="Label" value=t.label.clone().unwrap_or_else(|| "—".to_string()) />
                                    <DetailItem label="Error" value={
                                        if t.error_message.is_empty() { "None".to_string() } else { t.error_message.clone() }
                                    } />
                                </div>
                            </TabsContent>

                            <TabsContent value="transfer" class="flex-1 overflow-y-auto px-4 pb-3">
                                <div class="grid grid-cols-2 md:grid-cols-4 gap-x-6 gap-y-2 text-sm">
                                    <DetailItem label="Download Speed" value=format_speed(t.down_rate) />
                                    <DetailItem label="Upload Speed" value=format_speed(t.up_rate) />
                                    <DetailItem label="ETA" value=format_duration(t.eta) />
                                    <DetailItem label="Downloaded" value=format_bytes(t.completed) />
                                    <DetailItem label="Total Size" value=format_bytes(t.size) />
                                    <DetailItem label="Remaining" value=format_bytes(t.size - t.completed) />
                                </div>
                            </TabsContent>

                            <TabsContent value="files" class="flex-1 overflow-y-auto px-4 pb-3">
                                <div class="text-sm text-muted-foreground flex items-center gap-2 py-4">
                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
                                    </svg>
                                    "File list will be available when file API is connected."
                                </div>
                            </TabsContent>

                            <TabsContent value="peers" class="flex-1 overflow-y-auto px-4 pb-3">
                                <div class="text-sm text-muted-foreground flex items-center gap-2 py-4">
                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M18 18.72a9.094 9.094 0 003.741-.479 3 3 0 00-4.682-2.72m.94 3.198l.001.031c0 .225-.012.447-.037.666A11.944 11.944 0 0112 21c-2.17 0-4.207-.576-5.963-1.584A6.062 6.062 0 016 18.719m12 0a5.971 5.971 0 00-.941-3.197m0 0A5.995 5.995 0 0012 12.75a5.995 5.995 0 00-5.058 2.772m0 0a3 3 0 00-4.681 2.72 8.986 8.986 0 003.74.477m.94-3.197a5.971 5.971 0 00-.94 3.197M15 6.75a3 3 0 11-6 0 3 3 0 016 0zm6 3a2.25 2.25 0 11-4.5 0 2.25 2.25 0 014.5 0zm-13.5 0a2.25 2.25 0 11-4.5 0 2.25 2.25 0 014.5 0z" />
                                    </svg>
                                    "Peer list will be available when peer API is connected."
                                </div>
                            </TabsContent>
                        </Tabs>
                    </div>
                }
            }}
        </Show>
    }
}

#[component]
fn DetailItem(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
) -> impl IntoView {
    let title = value.clone();
    view! {
        <div class="flex flex-col gap-0.5 py-1">
            <span class="text-[10px] uppercase tracking-wider text-muted-foreground font-medium">{label}</span>
            <span class="text-foreground font-mono text-xs truncate" title=title>{value}</span>
        </div>
    }
}
