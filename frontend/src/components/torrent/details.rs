use leptos::prelude::*;
use crate::components::ui::sheet::*;
use crate::components::ui::tabs::*;
use crate::components::ui::skeleton::*;
use shared::Torrent;

#[component]
pub fn TorrentDetailsSheet() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    // Setup an effect to open the sheet when a torrent is selected
    Effect::new(move |_| {
        if store.selected_torrent.get().is_some() {
            if let Some(trigger) = document().get_element_by_id("torrent-details-trigger") {
                use wasm_bindgen::JsCast;
                let _ = trigger.dyn_into::<web_sys::HtmlElement>().map(|el| el.click());
            }
        }
    });

    let selected_torrent = Memo::new(move |_| {
        let hash = store.selected_torrent.get()?;
        store.torrents.with(|map| map.get(&hash).cloned())
    });

    view! {
        <Sheet>
            <SheetTrigger attr:id="torrent-details-trigger" class="hidden">""</SheetTrigger>
            <SheetContent 
                direction=SheetDirection::Bottom 
                class="h-[80vh] sm:h-[60vh] rounded-t-xl sm:rounded-t-2xl p-0 flex flex-col gap-0 border-t border-border shadow-2xl" 
                hide_close_button=true
            >
                <div class="px-6 py-4 border-b flex items-center justify-between sticky top-0 bg-card z-10">
                    <div class="flex flex-col gap-1 min-w-0">
                        <Show when=move || selected_torrent.get().is_some() fallback=move || view! { <Skeleton class="h-6 w-48" /> }>
                            <h2 class="font-bold text-lg truncate">
                                {move || selected_torrent.get().map(|t| t.name).unwrap_or_default()}
                            </h2>
                        </Show>
                        <Show when=move || selected_torrent.get().is_some() fallback=move || view! { <Skeleton class="h-4 w-24" /> }>
                            <p class="text-xs text-muted-foreground uppercase tracking-widest font-semibold flex items-center gap-2">
                                {move || selected_torrent.get().map(|t| format!("{:?}", t.status)).unwrap_or_default()}
                                <span class="bg-primary/20 text-primary px-1.5 py-0.5 rounded text-[10px] lowercase">{move || selected_torrent.get().map(|t| format!("{:.1}%", t.percent_complete)).unwrap_or_default()}</span>
                            </p>
                        </Show>
                    </div>
                    // Custom close button that also resets store.selected_torrent
                    <SheetClose class="rounded-full p-2 hover:bg-muted transition-colors border-none shadow-none cursor-pointer">
                        <icons::X class="size-5 opacity-70" on:click=move |_| store.selected_torrent.set(None) />
                    </SheetClose>
                </div>

                <div class="flex-1 min-h-0 overflow-hidden p-6 flex flex-col">
                    <Tabs default_value="general" class="flex-1 min-h-0 flex flex-col">
                        <TabsList class="w-full justify-start rounded-none border-b bg-transparent p-0">
                            <TabsTrigger value="general" class="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary rounded-none">
                                "Genel"
                            </TabsTrigger>
                            <TabsTrigger value="files" class="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary rounded-none">
                                "Dosyalar"
                            </TabsTrigger>
                            <TabsTrigger value="trackers" class="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary rounded-none">
                                "İzleyiciler"
                            </TabsTrigger>
                            <TabsTrigger value="peers" class="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary rounded-none">
                                "Eşler"
                            </TabsTrigger>
                        </TabsList>

                        <div class="relative flex-1 min-h-0 mt-4">
                            <div class="absolute inset-0 overflow-y-auto pb-12 pr-2">
                                <TabsContent value="general" class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
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
                                            <div class="flex flex-col gap-6">
                                                // Aktarım
                                                <div>
                                                    <h3 class="text-sm font-bold border-b pb-2 mb-4">"Aktarım"</h3>
                                                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                                        <InfoItem label="Geçen Süre" value="N/A".to_string() />
                                                        <InfoItem label="Kalan" value=format_duration(t.eta) />
                                                        <InfoItem label="Paylaşım Oranı" value=format!("{:.3}", t.ratio) />
                                                        <div class="hidden md:block"></div>
                                                        
                                                        <InfoItem label="İndirilen" value=format_bytes(t.completed) />
                                                        <InfoItem label="İndirme Hızı" value=format_speed(t.down_rate) class="text-blue-500" />
                                                        <InfoItem label="Boşa Giden" value=format_bytes(t.wasted) />
                                                        <div class="hidden md:block"></div>
                                                        
                                                        <InfoItem label="Gönderilen" value=format_bytes(t.uploaded) />
                                                        <InfoItem label="Gönderme Hızı" value=format_speed(t.up_rate) class="text-green-500" />
                                                        <div class="hidden md:block"></div>
                                                        <div class="hidden md:block"></div>
                                                        
                                                        <InfoItem label="Ortaklar" value="0 / 0 bağlı".to_string() />
                                                        <InfoItem label="Eşler" value="0 / 0 bağlı".to_string() />
                                                    </div>
                                                </div>

                                                // İzleyici
                                                <div>
                                                    <h3 class="text-sm font-bold border-b pb-2 mb-4">"İzleyici"</h3>
                                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                        <InfoItem label="İzleyici URL'si" value="İzleyiciler sekmesine bakınız".to_string() class="col-span-1 md:col-span-2 break-all text-xs" />
                                                        <InfoItem label="İzleyici Durumu" value="N/A".to_string() />
                                                    </div>
                                                </div>

                                                // Genel
                                                <div>
                                                    <h3 class="text-sm font-bold border-b pb-2 mb-4">"Genel"</h3>
                                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                        <InfoItem label="Kaydedilen Yer" value=t.save_path class="col-span-1 md:col-span-2 break-all font-mono text-xs" />
                                                        <InfoItem label="Boş Disk Alanı" value=format_bytes(t.free_disk_space) />
                                                        <InfoItem label="Oluşturulma Tarihi" value=format_date(t.added_date) />
                                                        <InfoItem label="Hash" value=t.hash class="col-span-1 md:col-span-2 break-all font-mono text-xs" />
                                                        <InfoItem label="Yorum" value="Yok".to_string() class="col-span-1 md:col-span-2 break-words text-xs" />
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
                                        <div class="flex flex-col items-center justify-center h-48 opacity-60">
                                            <icons::File class="size-12 mb-3 text-muted-foreground" />
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
                                        <div class="flex flex-col items-center justify-center h-48 opacity-60">
                                            <icons::Settings2 class="size-12 mb-3 text-muted-foreground" />
                                            <p class="text-sm font-medium">"İzleyici yükleniyor..."</p>
                                        </div>
                                    }),
                                }}
                            </TabsContent>

                            <TabsContent value="peers" class="h-full">
                                <div class="flex flex-col items-center justify-center h-48 opacity-60">
                                    <icons::Users class="size-12 mb-3 text-muted-foreground" />
                                    <p class="text-sm font-medium">"Eş listesi yakında eklenecek"</p>
                                </div>
                            </TabsContent>
                            </div>
                        </div>
                    </Tabs>
                </div>
            </SheetContent>
        </Sheet>
    }
}

#[component]
fn InfoItem(
    label: &'static str, 
    value: String, 
    #[prop(optional)] class: &'static str
) -> impl IntoView {
    view! {
        <div class=tailwind_fuse::tw_merge!("flex flex-col gap-1", class)>
            <span class="text-xs font-semibold text-muted-foreground uppercase opacity-80">{label}</span>
            <span class="text-sm font-medium">{value}</span>
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
