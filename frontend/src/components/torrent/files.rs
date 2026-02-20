use leptos::prelude::*;
use crate::components::ui::table::*;
use crate::components::ui::badge::*;
use crate::components::ui::shimmer::*;
use crate::components::ui::context_menu::*;
use shared::TorrentFile;

#[component]
pub fn TorrentFilesTab(hash: String) -> impl IntoView {
    let hash_clone = hash.clone();
    
    // Fetch files resource
    let files_resource = Resource::new(
        move || hash_clone.clone(),
        |h| async move { shared::server_fns::torrent::get_files(h).await.unwrap_or_default() }
    );

    // Refresh action
    let refresh_files = Action::new(|h: &String| {
        let h = h.clone();
        async move { shared::server_fns::torrent::get_files(h).await.unwrap_or_default() }
    });

    let stored_hash = StoredValue::new(hash);

    view! {
        <Suspense fallback=move || view! { <FilesFallback /> }>
            {move || {
                let files = match refresh_files.value().get() {
                    Some(f) => f,
                    None => files_resource.get().unwrap_or_default(),
                };

                if files.is_empty() {
                    return view! {
                        <div class="flex flex-col items-center justify-center h-48 opacity-60">
                            <icons::File class="size-12 mb-3 text-muted-foreground" />
                            <p class="text-sm font-medium">"Bu torrent için dosya bulunamadı."</p>
                        </div>
                    }.into_any();
                }

                let files_len = files.len();

                view! {
                    <div class="space-y-4">
                        <TableWrapper class="bg-card/50">
                            <Table>
                                <TableHeader class="sticky top-0 bg-muted/80 backdrop-blur-sm z-10">
                                    <TableRow class="hover:bg-transparent">
                                        <TableHead class="w-12 text-center text-xs">"#"</TableHead>
                                        <TableHead class="text-xs">"Dosya Adı"</TableHead>
                                        <TableHead class="w-24 text-right text-xs">"Boyut"</TableHead>
                                        <TableHead class="w-24 text-right text-xs">"Tamamlanan"</TableHead>
                                        <TableHead class="w-24 text-center text-xs">"Öncelik"</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    <For 
                                        each=move || files.clone()
                                        key=|f| f.index
                                        children={move |f| {
                                            let p_hash = stored_hash.get_value();
                                            let r_files = refresh_files.clone();
                                            view! {
                                                <FileRow 
                                                    file=f 
                                                    hash=p_hash
                                                    refresh_action=r_files
                                                />
                                            }
                                        }}
                                    />
                                </TableBody>
                            </Table>
                        </TableWrapper>
                        
                        <div class="flex items-center justify-between text-xs text-muted-foreground px-1">
                            <span>{format!("Toplam {} dosya", files_len)}</span>
                        </div>
                    </div>
                }.into_any()
            }}
        </Suspense>
    }
}

#[component]
fn FileRow(file: TorrentFile, hash: String, refresh_action: Action<String, Vec<TorrentFile>>) -> impl IntoView {
    let f_idx = file.index;
    let context_id = format!("file-context-{}-{}", hash, f_idx);
    let path_clone = file.path.clone();

    let set_priority = Action::new(|req: &(String, u32, u8)| {
        let (h, idx, p) = req.clone();
        async move {
            let res = shared::server_fns::torrent::set_file_priority(h, idx, p).await;
            if let Err(e) = &res {
                crate::store::show_toast(shared::NotificationLevel::Error, format!("Öncelik değiştirilemedi: {:?}", e));
            } else {
                crate::store::show_toast(shared::NotificationLevel::Success, "Dosya önceliği güncellendi.".to_string());
            }
            res
        }
    });

    view! {
        <ContextMenu>
            <ContextMenuTrigger attr:id=context_id.clone()>
                <TableRow class="hover:bg-muted/50 transition-colors group">
                    <TableCell class="text-center text-xs text-muted-foreground">{file.index}</TableCell>
                    <TableCell class="font-medium text-xs break-all max-w-[200px] md:max-w-md" attr:title=move || path_clone.clone()>
                        {file.path.clone()}
                    </TableCell>
                    <TableCell class="text-right text-xs text-muted-foreground whitespace-nowrap">
                        {format_bytes(file.size)}
                    </TableCell>
                    <TableCell class="text-right text-xs whitespace-nowrap">
                        <span class="text-primary font-medium">{format_bytes(file.completed_chunks)}</span>
                    </TableCell>
                    <TableCell class="text-center">
                        {
                            let (variant, label) = match file.priority {
                                0 => (BadgeVariant::Destructive, "İndirme"),
                                2 => (BadgeVariant::Success, "Yüksek"),
                                _ => (BadgeVariant::Secondary, "Normal"),
                            };
                            view! { <Badge variant=variant class="text-[10px] uppercase">{label}</Badge> }
                        }
                    </TableCell>
                </TableRow>
            </ContextMenuTrigger>

            <ContextMenuContent class="w-48">
                <ContextMenuLabel>"Dosya Önceliği"</ContextMenuLabel>
                <ContextMenuGroup>
                    <ContextMenuItem on:click={
                        let h = hash.clone();
                        let ra = refresh_action.clone();
                        move |_| {
                            set_priority.dispatch((h.clone(), f_idx, 2));
                            ra.dispatch(h.clone());
                        }
                    }>
                        <icons::ChevronsUp class="text-green-500" />
                        <span>"Yüksek"</span>
                    </ContextMenuItem>
                    
                    <ContextMenuItem on:click={
                        let h = hash.clone();
                        let ra = refresh_action.clone();
                        move |_| {
                            set_priority.dispatch((h.clone(), f_idx, 1));
                            ra.dispatch(h.clone());
                        }
                    }>
                        <icons::Minus class="text-blue-500" />
                        <span>"Normal"</span>
                    </ContextMenuItem>
                    
                    <ContextMenuItem class="text-destructive focus:bg-destructive/10" on:click={
                        let h = hash.clone();
                        let ra = refresh_action.clone();
                        move |_| {
                            set_priority.dispatch((h.clone(), f_idx, 0));
                            ra.dispatch(h.clone());
                        }
                    }>
                        <icons::X />
                        <span>"İndirme (Kapalı)"</span>
                    </ContextMenuItem>
                </ContextMenuGroup>
            </ContextMenuContent>
        </ContextMenu>
    }
}

#[component]
fn FilesFallback() -> impl IntoView {
    view! {
        <Shimmer loading=Signal::derive(|| true) class="space-y-2">
            <div class="h-10 w-full bg-muted/20 rounded-md"></div>
            <div class="h-10 w-full bg-muted/20 rounded-md"></div>
            <div class="h-10 w-full bg-muted/20 rounded-md"></div>
            <div class="h-10 w-full bg-muted/20 rounded-md"></div>
        </Shimmer>
    }
}

fn format_bytes(bytes: i64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes < 1024 { return format!("{} B", bytes); }
    let i = (bytes as f64).log2().div_euclid(10.0) as usize;
    format!("{:.1} {}", (bytes as f64) / 1024_f64.powi(i as i32), UNITS[i])
}
