use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashSet;
use icons::{ArrowUpDown, Inbox, Settings2, Play, Square, Trash2, Ellipsis, ArrowUp, ArrowDown, Check, ListFilter};
use crate::store::{get_action_messages, show_toast};
use crate::api;
use shared::NotificationLevel;
use crate::components::context_menu::TorrentContextMenu;
use crate::components::ui::data_table::*;
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::empty::*;
use crate::components::ui::input::Input;
use crate::components::ui::multi_select::*;
use crate::components::ui::dropdown_menu::*;
use crate::components::ui::alert_dialog::{
    AlertDialog,
    AlertDialogBody,
    AlertDialogClose,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
};
use tailwind_fuse::tw_merge;

const ALL_COLUMNS: [(&str, &str); 8] = [
    ("Name", "Name"),
    ("Size", "Size"),
    ("Progress", "Progress"),
    ("Status", "Status"),
    ("DownSpeed", "DL Speed"),
    ("UpSpeed", "UP Speed"),
    ("ETA", "ETA"),
    ("AddedDate", "Date"),
];

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
    if days > 0 { format!("{}d {}h", days, hours) }
    else if hours > 0 { format!("{}h {}m", hours, minutes) }
    else if minutes > 0 { format!("{}m {}s", minutes, secs) }
    else { format!("{}s", secs) }
}

fn format_date(timestamp: i64) -> String {
    if timestamp <= 0 { return "N/A".to_string(); }
    let dt = chrono::DateTime::from_timestamp(timestamp, 0);
    match dt { Some(dt) => dt.format("%d/%m/%Y %H:%M").to_string(), None => "N/A".to_string() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortColumn {
    Name, Size, Progress, Status, DownSpeed, UpSpeed, ETA, AddedDate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortDirection { Ascending, Descending }

#[component]
pub fn TorrentTable() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let sort_col = signal(SortColumn::AddedDate);
    let sort_dir = signal(SortDirection::Descending);
    
    let selected_hashes = RwSignal::new(HashSet::<String>::new());
    
    let visible_columns = RwSignal::new(HashSet::from([
        "Name".to_string(), "Size".to_string(), "Progress".to_string(), 
        "Status".to_string(), "DownSpeed".to_string(), "UpSpeed".to_string(),
        "ETA".to_string(), "AddedDate".to_string()
    ]));

    let sorted_hashes_data = Memo::new(move |_| {
        let torrents_map = store.torrents.get();
        let filter = store.filter.get();
        let search = store.search_query.get();
        let search_lower = search.to_lowercase();
        
        let mut torrents: Vec<shared::Torrent> = torrents_map.values().filter(|t| {
            let matches_filter = match filter {
                crate::store::FilterStatus::All => true,
                crate::store::FilterStatus::Downloading => t.status == shared::TorrentStatus::Downloading,
                crate::store::FilterStatus::Seeding => t.status == shared::TorrentStatus::Seeding,
                crate::store::FilterStatus::Completed => t.status == shared::TorrentStatus::Seeding || (t.status == shared::TorrentStatus::Paused && t.percent_complete >= 100.0),
                crate::store::FilterStatus::Paused => t.status == shared::TorrentStatus::Paused,
                crate::store::FilterStatus::Inactive => t.status == shared::TorrentStatus::Paused || t.status == shared::TorrentStatus::Error,
                _ => true,
            };
            let matches_search = if search_lower.is_empty() { true } else { t.name.to_lowercase().contains(&search_lower) };
            matches_filter && matches_search
        }).cloned().collect();

        torrents.sort_by(|a, b| {
            let col = sort_col.0.get();
            let dir = sort_dir.0.get();
            let cmp = match col {
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Size => a.size.cmp(&b.size),
                SortColumn::Progress => a.percent_complete.partial_cmp(&b.percent_complete).unwrap_or(std::cmp::Ordering::Equal),
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
            if dir == SortDirection::Descending { cmp.reverse() } else { cmp }
        });
        torrents
    });

    let filtered_hashes = Memo::new(move |_| {
        sorted_hashes_data.get().into_iter().map(|t| t.hash.clone()).collect::<Vec<String>>()
    });

    let selected_count = Signal::derive(move || {
        let current_hashes: HashSet<String> = filtered_hashes.get().into_iter().collect();
        selected_hashes.with(|selected| {
            selected.iter().filter(|h| current_hashes.contains(*h)).count()
        })
    });

    let has_selection = Signal::derive(move || selected_count.get() > 0);

    let handle_select_all = Callback::new(move |checked: bool| {
        selected_hashes.update(|selected| {
            let hashes = filtered_hashes.get_untracked();
            for h in hashes {
                if checked { selected.insert(h); } 
                else { selected.remove(&h); }
            }
        });
    });

    let handle_sort = move |col: SortColumn| {
        if sort_col.0.get() == col {
            sort_dir.1.update(|d| {
                *d = match d { SortDirection::Ascending => SortDirection::Descending, SortDirection::Descending => SortDirection::Ascending };
            });
        } else {
            sort_col.1.set(col);
            sort_dir.1.set(SortDirection::Ascending);
        }
    };

    let sort_icon = move |col: SortColumn| {
        let is_active = sort_col.0.get() == col;
        let class = if is_active { "size-3 text-primary" } else { "size-3 opacity-30 group-hover:opacity-100 transition-opacity" };
        view! { <ArrowUpDown class=class.to_string() /> }.into_any()
    };

    let bulk_action = move |action: &'static str| {
        let hashes: Vec<String> = selected_hashes.get().into_iter().collect();
        if hashes.is_empty() { return; }
        
        spawn_local(async move {
            let mut success = true;
            for hash in hashes {
                let res = match action {
                    "start" => api::torrent::start(&hash).await,
                    "stop" => api::torrent::stop(&hash).await,
                    "delete" => api::torrent::delete(&hash).await,
                    "delete_with_data" => api::torrent::delete_with_data(&hash).await,
                    _ => Ok(()),
                };
                if res.is_err() { success = false; }
            }
            if success {
                show_toast(NotificationLevel::Success, format!("Toplu işlem başarıyla tamamlandı: {}", action));
                selected_hashes.update(|s| s.clear());
            } else {
                show_toast(NotificationLevel::Error, "Bazı işlemler başarısız oldu.");
            }
        });
    };

    let on_action = Callback::new(move |(action, hash): (String, String)| {
        let (success_msg_str, error_msg_str): (&'static str, &'static str) = get_action_messages(&action);
        let success_msg = success_msg_str.to_string();
        let error_msg = error_msg_str.to_string();
        spawn_local(async move {
            let result = match action.as_str() {
                "delete" => api::torrent::delete(&hash).await,
                "delete_with_data" => api::torrent::delete_with_data(&hash).await,
                "start" => api::torrent::start(&hash).await,
                "stop" => api::torrent::stop(&hash).await,
                _ => api::torrent::action(&hash, &action).await,
            };
            match result {
                Ok(_) => show_toast(NotificationLevel::Success, success_msg),
                Err(e) => show_toast(NotificationLevel::Error, format!("{}: {:?}", error_msg, e)),
            }
        });
    });

    view! {
        <div class="h-full bg-background relative flex flex-col overflow-hidden px-4 py-4 gap-4">
            // --- TOPBAR ---
            <div class="flex items-center justify-between gap-4">
                <div class="flex items-center gap-2 flex-1 max-w-md">
                    <Input
                        class="h-9"
                        placeholder="Torrent ara..."
                        bind_value=store.search_query
                    />
                </div>

                <div class="flex items-center gap-2">
                    <Show when=move || has_selection.get()>
                        <DropdownMenu>
                            <DropdownMenuTrigger class="w-[140px] h-9 gap-2">
                                <Ellipsis class="size-4" />
                                {move || format!("Toplu İşlem ({})", selected_count.get())}
                            </DropdownMenuTrigger>
                            <DropdownMenuContent class="w-48">
                                <DropdownMenuLabel>"Seçili Torrentler"</DropdownMenuLabel>
                                <DropdownMenuGroup class="mt-2">
                                    <DropdownMenuItem on:click=move |_| bulk_action("start")>
                                        <Play class="mr-2 size-4" /> "Başlat"
                                    </DropdownMenuItem>
                                    <DropdownMenuItem on:click=move |_| bulk_action("stop")>
                                        <Square class="mr-2 size-4" /> "Durdur"
                                    </DropdownMenuItem>
                                    
                                    <div class="my-1 h-px bg-border" />
                                    
                                    <AlertDialog>
                                        <AlertDialogTrigger class="w-full text-left">
                                            <div class="inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm transition-colors text-destructive hover:bg-destructive/10 focus:bg-destructive/10 cursor-pointer">
                                                <Trash2 class="size-4" /> "Toplu Sil..."
                                            </div>
                                        </AlertDialogTrigger>
                                        <AlertDialogContent class="sm:max-w-[425px]">
                                            <AlertDialogBody>
                                                <AlertDialogHeader class="space-y-3">
                                                    <AlertDialogTitle class="text-destructive flex items-center gap-2 text-xl">
                                                        <Trash2 class="size-6" />
                                                        "Toplu Silme Onayı"
                                                    </AlertDialogTitle>
                                                    <AlertDialogDescription class="text-sm leading-relaxed">
                                                        {move || format!("Seçili {} adet torrent silinecek. Lütfen silme yöntemini seçin:", selected_count.get())}
                                                        <div class="mt-4 p-4 bg-destructive/5 rounded-lg border border-destructive/10 text-xs text-destructive/80 font-medium">
                                                            "⚠️ Dikkat: Verilerle birlikte silme işlemi dosyaları diskten de kalıcı olarak kaldıracaktır."
                                                        </div>
                                                    </AlertDialogDescription>
                                                </AlertDialogHeader>
                                                <AlertDialogFooter class="mt-6">
                                                    <div class="flex flex-col-reverse sm:flex-row gap-3 w-full sm:justify-end">
                                                        <AlertDialogClose class="sm:flex-1 md:flex-none">"Vazgeç"</AlertDialogClose>
                                                        <div class="flex flex-col sm:flex-row gap-2">
                                                            <Button 
                                                                variant=ButtonVariant::Secondary
                                                                class="w-full sm:w-auto font-medium"
                                                                on:click=move |_| bulk_action("delete")
                                                            >
                                                                "Sadece Sil"
                                                            </Button>
                                                            <Button 
                                                                variant=ButtonVariant::Destructive
                                                                class="w-full sm:w-auto font-bold"
                                                                on:click=move |_| bulk_action("delete_with_data")
                                                            >
                                                                "Verilerle Sil"
                                                            </Button>
                                                        </div>
                                                    </div>
                                                </AlertDialogFooter>
                                            </AlertDialogBody>
                                        </AlertDialogContent>
                                    </AlertDialog>
                                </DropdownMenuGroup>
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </Show>

                    // Mobile Sort Menu
                    <div class="block md:hidden">
                        <DropdownMenu>
                            <DropdownMenuTrigger class="w-[100px] h-9 gap-2 text-xs">
                                <ListFilter class="size-4" />
                                "Sırala"
                            </DropdownMenuTrigger>
                            <DropdownMenuContent class="w-56">
                                <DropdownMenuLabel>"Sıralama Ölçütü"</DropdownMenuLabel>
                                <DropdownMenuGroup class="mt-2">
                                    {move || {
                                        let current_col = sort_col.0.get();
                                        let current_dir = sort_dir.0.get();
                                        
                                        let sort_items = vec![
                                            (SortColumn::Name, "İsim"),
                                            (SortColumn::Size, "Boyut"),
                                            (SortColumn::Progress, "İlerleme"),
                                            (SortColumn::Status, "Durum"),
                                            (SortColumn::DownSpeed, "DL Hızı"),
                                            (SortColumn::UpSpeed, "UP Hızı"),
                                            (SortColumn::ETA, "Kalan Süre"),
                                            (SortColumn::AddedDate, "Tarih"),
                                        ];

                                        sort_items.into_iter().map(|(col, label)| {
                                            let is_active = current_col == col;
                                            view! {
                                                <DropdownMenuItem on:click=move |_| handle_sort(col)>
                                                    <div class="flex items-center justify-between w-full">
                                                        <div class="flex items-center gap-2">
                                                            {if is_active { view! { <Check class="size-4 text-primary" /> }.into_any() } else { view! { <div class="size-4" /> }.into_any() }}
                                                            <span class=if is_active { "font-bold text-primary" } else { "" }>{label}</span>
                                                        </div>
                                                        {if is_active {
                                                            match current_dir {
                                                                SortDirection::Ascending => view! { <ArrowUp class="size-3 opacity-50" /> }.into_any(),
                                                                SortDirection::Descending => view! { <ArrowDown class="size-3 opacity-50" /> }.into_any(),
                                                            }
                                                        } else { view! { "" }.into_any() }}
                                                    </div>
                                                </DropdownMenuItem>
                                            }.into_any()
                                        }).collect_view()
                                    }}
                                </DropdownMenuGroup>
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </div>

                    // Desktop Columns Menu
                    <div class="hidden md:flex">
                        <MultiSelect values=visible_columns>
                            <MultiSelectTrigger class="w-[140px] h-9">
                                <div class="flex items-center gap-2 text-xs">
                                    <Settings2 class="size-4" />
                                    "Sütunlar"
                                </div>
                            </MultiSelectTrigger>
                            <MultiSelectContent>
                                <MultiSelectGroup>
                                    {ALL_COLUMNS.into_iter().map(|(id, label)| {
                                        let id_val = id.to_string();
                                        view! {
                                            <MultiSelectItem>
                                                <MultiSelectOption value=id_val.clone() attr:disabled=move || id_val == "Name">
                                                    {label}
                                                </MultiSelectOption>
                                            </MultiSelectItem>
                                        }.into_any()
                                    }).collect_view()}
                                </MultiSelectGroup>
                            </MultiSelectContent>
                        </MultiSelect>
                    </div>
                </div>
            </div>

            // --- MAIN CONTENT ---
            <div class="flex-1 min-h-0 overflow-hidden">
                // Desktop Table View
                <DataTableWrapper class="hidden md:block h-full bg-card/50">
                    <div class="h-full overflow-auto">
                        <DataTable>
                            <DataTableHeader class="sticky top-0 bg-muted/80 backdrop-blur-sm z-10">
                                <DataTableRow class="hover:bg-transparent">
                                    <DataTableHead class="w-12 px-4">
                                        <Checkbox
                                            checked=Signal::derive(move || {
                                                let hashes = filtered_hashes.get();
                                                !hashes.is_empty() && selected_count.get() == hashes.len()
                                            })
                                            on_checked_change=handle_select_all
                                        />
                                    </DataTableHead>
                                    
                                    {move || visible_columns.get().contains("Name").then(|| view! {
                                        <DataTableHead class="cursor-pointer group select-none transition-all duration-100 active:scale-[0.98] hover:bg-muted/30 hover:text-foreground" on:click=move |_| handle_sort(SortColumn::Name)>
                                            <div class="flex items-center gap-2">"Name" {move || sort_icon(SortColumn::Name)}</div>
                                        </DataTableHead>
                                    }).into_any()}
                                    
                                    {move || visible_columns.get().contains("Size").then(|| view! {
                                        <DataTableHead class="w-24 cursor-pointer group select-none transition-all duration-100 active:scale-[0.98] hover:bg-muted/30 hover:text-foreground" on:click=move |_| handle_sort(SortColumn::Size)>
                                            <div class="flex items-center gap-2">"Size" {move || sort_icon(SortColumn::Size)}</div>
                                        </DataTableHead>
                                    }).into_any()}
                                    
                                    {move || visible_columns.get().contains("Progress").then(|| view! {
                                        <DataTableHead class="w-48 cursor-pointer group select-none transition-all duration-100 active:scale-[0.98] hover:bg-muted/30 hover:text-foreground" on:click=move |_| handle_sort(SortColumn::Progress)>
                                            <div class="flex items-center gap-2">"Progress" {move || sort_icon(SortColumn::Progress)}</div>
                                        </DataTableHead>
                                    }).into_any()}
                                    
                                    {move || visible_columns.get().contains("Status").then(|| view! {
                                        <DataTableHead class="w-24 cursor-pointer group select-none transition-all duration-100 active:scale-[0.98] hover:bg-muted/30 hover:text-foreground" on:click=move |_| handle_sort(SortColumn::Status)>
                                            <div class="flex items-center gap-2">"Status" {move || sort_icon(SortColumn::Status)}</div>
                                        </DataTableHead>
                                    }).into_any()}
                                    
                                    {move || visible_columns.get().contains("DownSpeed").then(|| view! {
                                        <DataTableHead class="w-24 cursor-pointer group select-none transition-all duration-100 active:scale-[0.98] hover:bg-muted/30 hover:text-foreground text-right" on:click=move |_| handle_sort(SortColumn::DownSpeed)>
                                            <div class="flex items-center justify-end gap-2">"DL Speed" {move || sort_icon(SortColumn::DownSpeed)}</div>
                                        </DataTableHead>
                                    }).into_any()}
                                    
                                    {move || visible_columns.get().contains("UpSpeed").then(|| view! {
                                        <DataTableHead class="w-24 cursor-pointer group select-none transition-all duration-100 active:scale-[0.98] hover:bg-muted/30 hover:text-foreground text-right" on:click=move |_| handle_sort(SortColumn::UpSpeed)>
                                            <div class="flex items-center justify-end gap-2">"UP Speed" {move || sort_icon(SortColumn::UpSpeed)}</div>
                                        </DataTableHead>
                                    }).into_any()}
                                    
                                    {move || visible_columns.get().contains("ETA").then(|| view! {
                                        <DataTableHead class="w-24 cursor-pointer group select-none transition-all duration-100 active:scale-[0.98] hover:bg-muted/30 hover:text-foreground text-right" on:click=move |_| handle_sort(SortColumn::ETA)>
                                            <div class="flex items-center justify-end gap-2">"ETA" {move || sort_icon(SortColumn::ETA)}</div>
                                        </DataTableHead>
                                    }).into_any()}
                                    
                                    {move || visible_columns.get().contains("AddedDate").then(|| view! {
                                        <DataTableHead class="w-32 cursor-pointer group select-none transition-all duration-100 active:scale-[0.98] hover:bg-muted/30 hover:text-foreground text-right" on:click=move |_| handle_sort(SortColumn::AddedDate)>
                                            <div class="flex items-center justify-end gap-2">"Date" {move || sort_icon(SortColumn::AddedDate)}</div>
                                        </DataTableHead>
                                    }).into_any()}
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody>
                                <Show
                                    when=move || !filtered_hashes.get().is_empty()
                                    fallback=move || view! {
                                        <DataTableRow class="hover:bg-transparent">
                                            <DataTableCell attr:colspan="10" class="h-[400px]">
                                                <Empty class="h-full">
                                                    <EmptyHeader>
                                                        <EmptyMedia variant=EmptyMediaVariant::Icon>
                                                            <Inbox class="size-10 text-muted-foreground" />
                                                        </EmptyMedia>
                                                        <EmptyTitle>"Torrent Bulunamadı"</EmptyTitle>
                                                        <EmptyDescription>
                                                            {move || {
                                                                let query = store.search_query.get();
                                                                if query.is_empty() { "Henüz torrent bulunmuyor.".to_string() }
                                                                else { "Arama kriterlerinize uygun sonuç bulunamadı.".to_string() }
                                                            }}
                                                        </EmptyDescription>
                                                    </EmptyHeader>
                                                </Empty>
                                            </DataTableCell>
                                        </DataTableRow>
                                    }.into_any()
                                >
                                    <For each=move || filtered_hashes.get() key=|hash| hash.clone() children={
                                        let on_action = on_action.clone();
                                        move |hash| {
                                            let h = hash.clone();
                                            let is_selected = Signal::derive(move || {
                                                selected_hashes.with(|selected| selected.contains(&h))
                                            });
                                            let h_for_change = hash.clone();
                                            view! { 
                                                <TorrentRow 
                                                    hash=hash.clone() 
                                                    on_action=on_action.clone() 
                                                    is_selected=is_selected
                                                    visible_columns=visible_columns
                                                    on_select=Callback::new(move |checked| {
                                                        selected_hashes.update(|selected| {
                                                            if checked { selected.insert(h_for_change.clone()); }
                                                            else { selected.remove(&h_for_change); }
                                                        });
                                                    })
                                                /> 
                                            }
                                        }
                                    } />
                                </Show>
                            </DataTableBody>
                        </DataTable>
                    </div>
                </DataTableWrapper>

                // Mobile Card View
                <div class="block md:hidden h-full overflow-y-auto space-y-4 pb-32 px-1">
                     <Show
                        when=move || !filtered_hashes.get().is_empty()
                        fallback=move || view! {
                            <div class="flex flex-col items-center justify-center h-64 opacity-50 text-muted-foreground">
                                <Inbox class="size-12 mb-2" />
                                <p>"Torrent Bulunamadı"</p>
                            </div>
                        }.into_any()
                    >
                        <For each=move || filtered_hashes.get() key=|hash| hash.clone() children={
                            let on_action = on_action.clone();
                            move |hash| {
                                let h = hash.clone();
                                let is_selected = Signal::derive(move || {
                                    selected_hashes.with(|selected| selected.contains(&h))
                                });
                                let h_for_change = hash.clone();
                                
                                view! { 
                                    <TorrentCard 
                                        hash=hash.clone() 
                                        on_action=on_action.clone() 
                                        is_selected=is_selected
                                        on_select=Callback::new(move |checked| {
                                            selected_hashes.update(|selected| {
                                                if checked { selected.insert(h_for_change.clone()); }
                                                else { selected.remove(&h_for_change); }
                                            });
                                        })
                                    /> 
                                }
                            }
                        } />
                    </Show>
                </div>
            </div>

            <div class="hidden md:flex items-center justify-between px-2 py-1 text-[11px] text-muted-foreground bg-muted/20 border rounded-md">
                <div class="flex gap-4">
                    <span>{move || format!("Toplam: {} torrent", filtered_hashes.get().len())}</span>
                    <Show when=move || has_selection.get()>
                        <span class="text-primary font-medium">{move || format!("{} torrent seçili", selected_count.get())}</span>
                    </Show>
                </div>
                <div>"VibeTorrent v3"</div>
            </div>
        </div>
    }.into_any()
}

#[component]
fn TorrentRow(
    hash: String,
    on_action: Callback<(String, String)>,
    is_selected: Signal<bool>,
    visible_columns: RwSignal<HashSet<String>>,
    on_select: Callback<bool>,
) -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let h = hash.clone();
    let torrent = Memo::new(move |_| store.torrents.with(|map| map.get(&h).cloned()));

    let stored_hash = StoredValue::new(hash.clone());

    view! {
        <Show when=move || torrent.get().is_some() fallback=|| ()>
            {
                let on_action = on_action.clone();
                move || {
                    let t = torrent.get().unwrap();
                    let t_name = t.name.clone();
                    
                    let is_active_selection = Memo::new(move |_| {
                        let selected = store.selected_torrent.get();
                        selected.as_deref() == Some(stored_hash.get_value().as_str())
                    });

                    let t_name_stored = StoredValue::new(t_name.clone());
                    let h_for_menu = stored_hash.get_value();

                    view! {
                        <TorrentContextMenu torrent_hash=h_for_menu on_action=on_action.clone()>
                            <DataTableRow 
                                class="cursor-pointer group h-10"
                                attr:data-state=move || if is_selected.get() || is_active_selection.get() { "selected" } else { "" }
                                on:click=move |_| store.selected_torrent.set(Some(stored_hash.get_value()))
                            >
                                <DataTableCell class="w-12 px-4">
                                    <Checkbox 
                                        checked=is_selected 
                                        on_checked_change=on_select 
                                    />
                                </DataTableCell>
                                
                                {move || visible_columns.get().contains("Name").then({
                                    move || view! {
                                        <DataTableCell class="font-medium truncate max-w-[200px] lg:max-w-md" attr:title=t_name_stored.get_value()>
                                            {t_name_stored.get_value()}
                                        </DataTableCell>
                                    }
                                }).into_any()}

                                {move || visible_columns.get().contains("Size").then({
                                    let size_bytes = t.size;
                                    move || {
                                        let size_str = format_bytes(size_bytes);
                                        view! { <DataTableCell class="font-mono text-xs text-muted-foreground whitespace-nowrap">{size_str}</DataTableCell> }
                                    }
                                }).into_any()}

                                {move || visible_columns.get().contains("Progress").then({
                                    let percent = t.percent_complete;
                                    move || view! {
                                        <DataTableCell>
                                            <div class="flex items-center gap-2">
                                                <div class="h-1.5 w-full bg-secondary rounded-full overflow-hidden min-w-[80px]">
                                                    <div class="h-full bg-primary transition-all duration-500" style=format!("width: {}%", percent)></div>
                                                </div>
                                                <span class="text-[10px] text-muted-foreground w-10 text-right">{format!("{:.1}%", percent)}</span>
                                            </div>
                                        </DataTableCell>
                                    }
                                }).into_any()}

                                {move || visible_columns.get().contains("Status").then({
                                    let status_text = format!("{:?}", t.status);
                                    let variant = match t.status {
                                        shared::TorrentStatus::Seeding => BadgeVariant::Success,
                                        shared::TorrentStatus::Downloading => BadgeVariant::Info,
                                        shared::TorrentStatus::Paused => BadgeVariant::Warning,
                                        shared::TorrentStatus::Error => BadgeVariant::Destructive,
                                        _ => BadgeVariant::Secondary,
                                    };
                                    move || view! { 
                                        <DataTableCell class="whitespace-nowrap">
                                            <Badge variant=variant>{status_text.clone()}</Badge>
                                        </DataTableCell> 
                                    }
                                }).into_any()}

                                {move || visible_columns.get().contains("DownSpeed").then({
                                    let rate = t.down_rate;
                                    move || {
                                        let speed_str = format_speed(rate);
                                        view! { <DataTableCell class="text-right font-mono text-xs text-green-600 dark:text-green-500 whitespace-nowrap">{speed_str}</DataTableCell> }
                                    }
                                }).into_any()}

                                {move || visible_columns.get().contains("UpSpeed").then({
                                    let rate = t.up_rate;
                                    move || {
                                        let speed_str = format_speed(rate);
                                        view! { <DataTableCell class="text-right font-mono text-xs text-blue-600 dark:text-blue-500 whitespace-nowrap">{speed_str}</DataTableCell> }
                                    }
                                }).into_any()}

                                {move || visible_columns.get().contains("ETA").then({
                                    let eta = t.eta;
                                    move || {
                                        let eta_str = format_duration(eta);
                                        view! { <DataTableCell class="text-right font-mono text-xs text-muted-foreground whitespace-nowrap">{eta_str}</DataTableCell> }
                                    }
                                }).into_any()}

                                {move || visible_columns.get().contains("AddedDate").then({
                                    let date = t.added_date;
                                    move || {
                                        let date_str = format_date(date);
                                        view! { <DataTableCell class="text-right font-mono text-xs text-muted-foreground whitespace-nowrap">{date_str}</DataTableCell> }
                                    }
                                }).into_any()}
                            </DataTableRow>
                        </TorrentContextMenu>
                    }.into_any()
                }
            }
        </Show>
    }.into_any()
}

#[component]
fn TorrentCard(
    hash: String,
    on_action: Callback<(String, String)>,
    is_selected: Signal<bool>,
    on_select: Callback<bool>,
) -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let h = hash.clone();
    let torrent = Memo::new(move |_| store.torrents.with(|map| map.get(&h).cloned()));

    let stored_hash = StoredValue::new(hash.clone());

    view! {
        <Show when=move || torrent.get().is_some() fallback=|| ()>
            {
                let on_action = on_action.clone();
                move || {
                    let t = torrent.get().unwrap();
                    let t_name = t.name.clone();
                    let status_variant = match t.status { 
                        shared::TorrentStatus::Seeding => BadgeVariant::Success, 
                        shared::TorrentStatus::Downloading => BadgeVariant::Info, 
                        shared::TorrentStatus::Paused => BadgeVariant::Warning, 
                        shared::TorrentStatus::Error => BadgeVariant::Destructive, 
                        _ => BadgeVariant::Secondary 
                    };
                    let h_for_menu = stored_hash.get_value();

                    view! {
                        <TorrentContextMenu torrent_hash=h_for_menu on_action=on_action.clone()>
                            <div
                                class=move || tw_merge!(
                                    "rounded-lg transition-all duration-200 border cursor-pointer select-none overflow-hidden active:scale-[0.98]",
                                    if is_selected.get() { 
                                        "bg-primary/10 border-primary shadow-sm" 
                                    } else { 
                                        "bg-card border-border hover:border-primary/50" 
                                    }
                                )
                                on:click=move |_| {
                                    let current = is_selected.get();
                                    on_select.run(!current);
                                    store.selected_torrent.set(Some(stored_hash.get_value()));
                                }
                            >
                                <div class="p-4 space-y-3">
                                    <div class="flex justify-between items-start gap-3">
                                        <div class="flex-1 min-w-0">
                                            <h3 class="text-sm font-bold leading-tight line-clamp-2 break-all">{t_name.clone()}</h3>
                                        </div>
                                        <Badge variant=status_variant class="uppercase tracking-wider text-[10px]">
                                            {format!("{:?}", t.status)}
                                        </Badge>
                                    </div>

                                    <div class="space-y-1.5">
                                        <div class="flex justify-between text-[10px] font-medium text-muted-foreground">
                                            <span class="flex items-center gap-1">
                                                <span class="opacity-70">"Boyut:"</span> {format_bytes(t.size)}
                                            </span>
                                            <span class="font-bold text-primary">{format!("{:.1}%", t.percent_complete)}</span>
                                        </div>
                                        <div class="h-2 w-full bg-secondary rounded-full overflow-hidden">
                                            <div class="h-full bg-primary transition-all duration-500 ease-out" style=format!("width: {}%", t.percent_complete)></div>
                                        </div>
                                    </div>

                                    <div class="grid grid-cols-2 gap-y-2 gap-x-4 text-[10px] font-mono pt-2 border-t border-border/40">
                                        <div class="flex flex-col gap-0.5">
                                            <span class="text-muted-foreground uppercase text-[8px] tracking-tighter">"İndirme"</span>
                                            <span class="text-blue-600 dark:text-blue-400 font-bold">{format_speed(t.down_rate)}</span>
                                        </div>
                                        <div class="flex flex-col gap-0.5">
                                            <span class="text-muted-foreground uppercase text-[8px] tracking-tighter">"Gönderme"</span>
                                            <span class="text-green-600 dark:text-green-400 font-bold">{format_speed(t.up_rate)}</span>
                                        </div>
                                        <div class="flex flex-col gap-0.5">
                                            <span class="text-muted-foreground uppercase text-[8px] tracking-tighter">"Kalan Süre"</span>
                                            <span class="text-foreground font-medium">{format_duration(t.eta)}</span>
                                        </div>
                                        <div class="flex flex-col gap-0.5 items-end text-right">
                                            <span class="text-muted-foreground uppercase text-[8px] tracking-tighter">"Eklenme"</span>
                                            <span class="text-foreground/70">{format_date(t.added_date)}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </TorrentContextMenu>
                    }.into_any()
                }
            }
        </Show>
    }.into_any()
}
