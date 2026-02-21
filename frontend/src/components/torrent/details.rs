use leptos::prelude::*;
use crate::components::ui::tabs::*;
use crate::components::ui::skeleton::*;

#[component]
pub fn TorrentDetailsPanel() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    let selected_torrent = Memo::new(move |_| {
        let hash = store.selected_torrent.get()?;
        store.torrents.with(|map| map.get(&hash).cloned())
    });

    let is_open = Signal::derive(move || store.selected_torrent.get().is_some());

    view! {
        // Mobil overlay backdrop
        <div
            class=move || if is_open.get() {
                "fixed inset-0 bg-black/40 z-30 md:hidden backdrop-blur-sm transition-opacity duration-300 opacity-100"
            } else {
                "fixed inset-0 bg-black/0 z-30 md:hidden pointer-events-none transition-opacity duration-300 opacity-0"
            }
            on:click=move |_| store.selected_torrent.set(None)
        />

        // Panel — masaüstünde sağ kolonda sabit, mobilde sağdan açılan overlay
        <div class=move || {
            if is_open.get() {
                // Açık: masaüstünde görünür, mobilde sağdan gelir
                "w-full md:w-[380px] md:min-w-[380px] shrink-0 \
                 flex flex-col border-l border-border bg-card \
                 fixed top-0 right-0 bottom-0 z-40 \
                 translate-x-0 \
                 md:static md:z-auto md:translate-x-0 \
                 transition-transform duration-300 ease-out shadow-2xl md:shadow-none"
            } else {
                // Kapalı: masaüstünde gizli, mobilde sağa kayar
                "w-full md:w-0 shrink-0 overflow-hidden border-none \
                 fixed top-0 right-0 bottom-0 z-40 \
                 translate-x-full \
                 md:static md:z-auto md:translate-x-0 \
                 transition-transform duration-300 ease-in pointer-events-none"
            }
        }>
            // İpucu: panel kapalıyken içeriği render etme
            <Show when=move || is_open.get()>
                // Başlık
                <div class="px-4 py-3 border-b flex items-center justify-between shrink-0 bg-card">
                    <div class="flex flex-col gap-0.5 min-w-0 flex-1">
                        <Show
                            when=move || selected_torrent.get().is_some()
                            fallback=move || view! { <Skeleton class="h-5 w-40" /> }
                        >
                            <h2 class="font-bold text-sm truncate leading-tight">
                                {move || selected_torrent.get().map(|t| t.name).unwrap_or_default()}
                            </h2>
                        </Show>
                        <Show
                            when=move || selected_torrent.get().is_some()
                            fallback=move || view! { <Skeleton class="h-3 w-20 mt-1" /> }
                        >
                            <p class="text-[10px] text-muted-foreground uppercase tracking-widest font-semibold flex items-center gap-1.5">
                                {move || selected_torrent.get().map(|t| format!("{:?}", t.status)).unwrap_or_default()}
                                <span class="bg-primary/20 text-primary px-1 py-0.5 rounded text-[9px] lowercase">
                                    {move || selected_torrent.get().map(|t| format!("{:.1}%", t.percent_complete)).unwrap_or_default()}
                                </span>
                            </p>
                        </Show>
                    </div>
                    // Kapat butonu
                    <button
                        class="rounded-full p-1.5 hover:bg-muted transition-colors text-muted-foreground hover:text-foreground shrink-0 ml-2"
                        on:click=move |_| store.selected_torrent.set(None)
                    >
                        <icons::X class="size-4" />
                    </button>
                </div>

                // Sekmeler + içerik
                <div class="flex-1 overflow-hidden flex flex-col min-h-0">
                    <Tabs default_value="general" class="flex-1 h-full min-h-0 flex flex-col">
                        <TabsList class="w-full justify-start rounded-none border-b bg-transparent p-0 shrink-0 px-2">
                            <TabsTrigger
                                value="general"
                                class="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary rounded-none text-xs h-9"
                            >
                                "Genel"
                            </TabsTrigger>
                            <TabsTrigger
                                value="files"
                                class="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary rounded-none text-xs h-9"
                            >
                                "Dosyalar"
                            </TabsTrigger>
                            <TabsTrigger
                                value="trackers"
                                class="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary rounded-none text-xs h-9"
                            >
                                "İzleyiciler"
                            </TabsTrigger>
                            <TabsTrigger
                                value="peers"
                                class="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary rounded-none text-xs h-9"
                            >
                                "Eşler"
                            </TabsTrigger>
                        </TabsList>

                        <crate::components::ui::scroll_area::ScrollArea class="flex-1 min-h-0">
                            <TabsContent value="general" class="p-4 space-y-5 animate-in fade-in duration-200">
                                <crate::components::ui::shimmer::Shimmer
                                    loading=Signal::derive(move || selected_torrent.get().is_none())
                                    shimmer_color="rgba(0,0,0,0.06)"
                                    background_color="rgba(0,0,0,0.04)"
                                >
                                    {move || {
                                        let t = selected_torrent.get().unwrap_or_else(|| shared::Torrent {
                                            hash: "----------------------------------------".to_string(),
                                            name: "Yükleniyor...".to_string(),
                                            size: 0,
                                            completed: 0,
                                            down_rate: 0,
                                            up_rate: 0,
                                            eta: 0,
                                            percent_complete: 0.0,
                                            status: shared::TorrentStatus::Downloading,
                                            error_message: "".to_string(),
                                            added_date: 0,
                                            label: None,
                                            ratio: 0.0,
                                            uploaded: 0,
                                            wasted: 0,
                                            save_path: "Yükleniyor...".to_string(),
                                            free_disk_space: 0,
                                        });

                                        view! {
                                            <div class="flex flex-col gap-5">
                                                // Aktarım
                                                <div>
                                                    <h3 class="text-[10px] font-bold border-b pb-1.5 mb-3 uppercase tracking-widest text-muted-foreground">"Aktarım"</h3>
                                                    <div class="grid grid-cols-2 gap-3">
                                                        <InfoItem label="Kalan" value=format_duration(t.eta) />
                                                        <InfoItem label="Paylaşım Oranı" value=format!("{:.3}", t.ratio) />
                                                        <InfoItem label="İndirilen" value=format_bytes(t.completed) />
                                                        <InfoItem label="İndirme Hızı" value=format_speed(t.down_rate) class="text-blue-500" />
                                                        <InfoItem label="Gönderilen" value=format_bytes(t.uploaded) />
                                                        <InfoItem label="Gönderme Hızı" value=format_speed(t.up_rate) class="text-green-500" />
                                                        <InfoItem label="Boşa Giden" value=format_bytes(t.wasted) />
                                                    </div>
                                                </div>

                                                // Genel
                                                <div>
                                                    <h3 class="text-[10px] font-bold border-b pb-1.5 mb-3 uppercase tracking-widest text-muted-foreground">"Genel"</h3>
                                                    <div class="flex flex-col gap-3">
                                                        <InfoItem label="Kaydedilen Yer" value=t.save_path class="break-all font-mono text-xs" />
                                                        <div class="grid grid-cols-2 gap-3">
                                                            <InfoItem label="Boş Disk Alanı" value=format_bytes(t.free_disk_space) />
                                                            <InfoItem label="Oluşturulma Tarihi" value=format_date(t.added_date) />
                                                        </div>
                                                        <InfoItem label="Hash" value=t.hash class="break-all font-mono text-[10px]" />
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }}
                                </crate::components::ui::shimmer::Shimmer>
                            </TabsContent>

                            <TabsContent value="files" class="h-full">
                                {move || match selected_torrent.get() {
                                    Some(t) => leptos::either::Either::Left(view! {
                                        <div class="h-full">
                                            <crate::components::torrent::files::TorrentFilesTab hash=t.hash />
                                        </div>
                                    }),
                                    None => leptos::either::Either::Right(view! {
                                        <div class="flex flex-col items-center justify-center h-48 opacity-60 gap-2">
                                            <icons::File class="size-10 text-muted-foreground" />
                                            <p class="text-sm font-medium">"Dosya yükleniyor..."</p>
                                        </div>
                                    }),
                                }}
                            </TabsContent>

                            <TabsContent value="trackers" class="h-full">
                                {move || match selected_torrent.get() {
                                    Some(t) => leptos::either::Either::Left(view! {
                                        <div class="h-full">
                                            <crate::components::torrent::trackers::TorrentTrackersTab hash=t.hash />
                                        </div>
                                    }),
                                    None => leptos::either::Either::Right(view! {
                                        <div class="flex flex-col items-center justify-center h-48 opacity-60 gap-2">
                                            <icons::Settings2 class="size-10 text-muted-foreground" />
                                            <p class="text-sm font-medium">"İzleyici yükleniyor..."</p>
                                        </div>
                                    }),
                                }}
                            </TabsContent>

                            <TabsContent value="peers" class="h-full">
                                <div class="flex flex-col items-center justify-center h-48 opacity-60 gap-2">
                                    <icons::Users class="size-10 text-muted-foreground" />
                                    <p class="text-sm font-medium">"Eş listesi yakında eklenecek"</p>
                                </div>
                            </TabsContent>
                        </crate::components::ui::scroll_area::ScrollArea>
                    </Tabs>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn InfoItem(
    label: &'static str,
    value: String,
    #[prop(optional)] class: &'static str
) -> impl IntoView {
    view! {
        <div class=tailwind_fuse::tw_merge!("flex flex-col gap-0.5", class)>
            <span class="text-[9px] font-semibold text-muted-foreground uppercase tracking-wider opacity-70">{label}</span>
            <span class="text-xs font-medium leading-tight">{value}</span>
        </div>
    }
}

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
    if days > 0 { format!("{}g {}s", days, hours) }
    else if hours > 0 { format!("{}s {}d", hours, minutes) }
    else if minutes > 0 { format!("{}d {}sn", minutes, secs) }
    else { format!("{}sn", secs) }
}

fn format_date(timestamp: i64) -> String {
    if timestamp <= 0 { return "N/A".to_string(); }
    let dt = chrono::DateTime::from_timestamp(timestamp, 0);
    match dt { Some(dt) => dt.format("%d/%m/%Y %H:%M").to_string(), None => "N/A".to_string() }
}
