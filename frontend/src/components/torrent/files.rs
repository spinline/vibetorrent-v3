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

    // Callback to trigger a refetch — safe, doesn't destroy existing components
    let on_refresh = Callback::new(move |_: ()| {
        files_resource.refetch();
    });

    let stored_hash = StoredValue::new(hash);

    view! {
        <Suspense fallback=move || view! { <FilesFallback /> }>
            {move || {
                let files = files_resource.get().unwrap_or_default();

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
                                            view! {
                                                <FileRow 
                                                    file=f 
                                                    hash=p_hash
                                                    on_refresh=on_refresh.clone()
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
fn FileRow(file: TorrentFile, hash: String, on_refresh: Callback<()>) -> impl IntoView {
    let f_idx = file.index;
    let path_clone = file.path.clone();

    // on_refresh is called AFTER the server responds, not before
    let on_refresh_stored = StoredValue::new(on_refresh);

    let set_priority = Action::new(move |req: &(String, u32, u8)| {
        let (h, idx, p) = req.clone();
        async move {
            let res = shared::server_fns::torrent::set_file_priority(h, idx, p).await;
            if let Err(e) = &res {
                crate::store::show_toast(shared::NotificationLevel::Error, format!("Öncelik değiştirilemedi: {:?}", e));
            } else {
                crate::store::show_toast(shared::NotificationLevel::Success, "Dosya önceliği güncellendi.".to_string());
                // Refetch AFTER the server has saved the priority
                on_refresh_stored.get_value().run(());
            }
            res
        }
    });

    view! {
        <FileContextMenu 
            torrent_hash=hash 
            file_index=f_idx 
            set_priority=set_priority
        >
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
        </FileContextMenu>
    }
}

#[component]
fn FileContextMenu(
    children: Children,
    torrent_hash: String,
    file_index: u32,
    set_priority: Action<(String, u32, u8), Result<(), ServerFnError>>,
) -> impl IntoView {
    let hash_c1 = torrent_hash.clone();
    let hash_c2 = torrent_hash.clone();
    let hash_c3 = torrent_hash.clone();

    view! {
        <ContextMenu>
            <ContextMenuTrigger>
                {children()}
            </ContextMenuTrigger>

            <ContextMenuContent class="w-48">
                <ContextMenuLabel>"Dosya Önceliği"</ContextMenuLabel>
                <ContextMenuGroup>
                    <ContextMenuItem on:click={
                        let h = hash_c1;
                        let sp = set_priority.clone();
                        move |_| {
                            sp.dispatch((h.clone(), file_index, 2));
                            crate::components::ui::context_menu::close_context_menu();
                        }
                    }>
                        <icons::ChevronsUp class="text-green-500" />
                        <span>"Yüksek"</span>
                    </ContextMenuItem>
                    
                    <ContextMenuItem on:click={
                        let h = hash_c2;
                        let sp = set_priority.clone();
                        move |_| {
                            sp.dispatch((h.clone(), file_index, 1));
                            crate::components::ui::context_menu::close_context_menu();
                        }
                    }>
                        <icons::Minus class="text-blue-500" />
                        <span>"Normal"</span>
                    </ContextMenuItem>
                    
                    <ContextMenuItem class="text-destructive focus:bg-destructive/10" on:click={
                        let h = hash_c3;
                        let sp = set_priority.clone();
                        move |_| {
                            sp.dispatch((h.clone(), file_index, 0));
                            crate::components::ui::context_menu::close_context_menu();
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
