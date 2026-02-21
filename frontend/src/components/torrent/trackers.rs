use leptos::prelude::*;
use crate::components::ui::table::*;
use crate::components::ui::shimmer::*;
use shared::TorrentTracker;

#[component]
pub fn TorrentTrackersTab(hash: String) -> impl IntoView {
    let hash_clone = hash.clone();
    
    let trackers_resource = Resource::new(
        move || hash_clone.clone(),
        |h| async move { shared::server_fns::torrent::get_trackers(h).await.unwrap_or_default() }
    );

    view! {
        <Suspense fallback=move || view! { <TrackersFallback /> }>
            {move || {
                let trackers = trackers_resource.get().unwrap_or_default();

                if trackers.is_empty() {
                    return view! {
                        <div class="flex flex-col items-center justify-center h-48 opacity-60">
                            <icons::Settings2 class="size-12 mb-3 text-muted-foreground" />
                            <p class="text-sm font-medium">"Bu torrent için izleyici bulunamadı."</p>
                        </div>
                    }.into_any();
                }

                view! {
                    <div class="h-full overflow-auto">
                        <TableWrapper class="bg-card/50 whitespace-nowrap">
                            <Table>
                                <TableHeader class="sticky top-0 bg-muted/80 backdrop-blur-sm z-10">
                                    <TableRow class="hover:bg-transparent text-xs">
                                        <TableHead>"İsim"</TableHead>
                                        <TableHead class="text-center">"Tür"</TableHead>
                                        <TableHead class="text-center">"Etkin"</TableHead>
                                        <TableHead class="text-center">"Grup"</TableHead>
                                        <TableHead class="text-center">"Ortaklar"</TableHead>
                                        <TableHead class="text-center">"Eşler"</TableHead>
                                        <TableHead class="text-center">"İndirilen"</TableHead>
                                        <TableHead class="text-center">"Son Güncelleme"</TableHead>
                                        <TableHead class="text-center">"Sıklık"</TableHead>
                                        <TableHead class="text-center">"Özel"</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    <For 
                                        each=move || trackers.clone()
                                        key=|t| t.url.clone()
                                        children=move |t| {
                                            let t_type = if t.url.starts_with("http") { "http" } 
                                                         else if t.url.starts_with("udp") { "udp" } 
                                                         else if t.url.starts_with("dht") { "dht" } 
                                                         else { "diğer" };
                                            let is_enabled = if t.is_enabled { "evet" } else { "hayır" };
                                            
                                            // Format timestamp difference for last update
                                            let now = chrono::Utc::now().timestamp();
                                            let diff = now - t.last_updated;
                                            let last_update_str = if t.last_updated == 0 { 
                                                "Güncellenmedi".to_string() 
                                            } else if diff >= 0 {
                                                format_duration_short(diff)
                                            } else {
                                                "N/A".to_string()
                                            };
                                            
                                            let url_clone = t.url.clone();
                                            view! {
                                                <TableRow class="hover:bg-muted/50 transition-colors group text-xs text-muted-foreground">
                                                    <TableCell class="font-medium text-foreground max-w-[200px] md:max-w-md truncate" attr:title=url_clone>
                                                        {t.url.clone()}
                                                    </TableCell>
                                                    <TableCell class="text-center">{t_type}</TableCell>
                                                    <TableCell class="text-center">{is_enabled}</TableCell>
                                                    <TableCell class="text-center">{t.group}</TableCell>
                                                    <TableCell class="text-center">{t.seeders}</TableCell>
                                                    <TableCell class="text-center">{t.peers}</TableCell>
                                                    <TableCell class="text-center">{t.downloaded}</TableCell>
                                                    <TableCell class="text-center">{last_update_str}</TableCell>
                                                    <TableCell class="text-center">{format_duration_short(t.interval)}</TableCell>
                                                    <TableCell class="text-center">"bilinmiyor"</TableCell> // Özel flag isn't cleanly via XMLRPC per tracker usually
                                                </TableRow>
                                            }
                                        }
                                    />
                                </TableBody>
                            </Table>
                        </TableWrapper>
                    </div>
                }.into_any()
            }}
        </Suspense>
    }
}

#[component]
fn TrackersFallback() -> impl IntoView {
    view! {
        <Shimmer loading=Signal::derive(|| true) class="space-y-2">
            <div class="h-10 w-full bg-muted/20 rounded-md"></div>
            <div class="h-10 w-full bg-muted/20 rounded-md"></div>
            <div class="h-10 w-full bg-muted/20 rounded-md"></div>
            <div class="h-10 w-full bg-muted/20 rounded-md"></div>
        </Shimmer>
    }
}

fn format_duration_short(seconds: i64) -> String {
    if seconds <= 0 { return "0sn".to_string(); }
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if days > 0 { format!("{}g {}s", days, hours) }
    else if hours > 0 { format!("{}s {}dk", hours, minutes) }
    else if minutes > 0 { format!("{}dk {}sn", minutes, secs) }
    else { format!("{}sn", secs) }
}
