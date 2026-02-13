use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::components::ui::sidenav::*;
use crate::components::ui::button::{Button, ButtonVariant, ButtonSize};
use crate::components::ui::theme_toggle::ThemeToggle;
use crate::components::ui::switch::Switch;

#[component]
pub fn Sidebar() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    let total_count = move || store.torrents.with(|map| map.len());
    let downloading_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| t.status == shared::TorrentStatus::Downloading)
                .count()
        })
    };
    let seeding_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| t.status == shared::TorrentStatus::Seeding)
                .count()
        })
    };
    let completed_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| {
                    t.status == shared::TorrentStatus::Seeding
                        || (t.status == shared::TorrentStatus::Paused && t.percent_complete >= 100.0)
                })
                .count()
        })
    };
    let paused_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| t.status == shared::TorrentStatus::Paused)
                .count()
        })
    };
    let inactive_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| {
                    t.status == shared::TorrentStatus::Paused
                        || t.status == shared::TorrentStatus::Error
                })
                .count()
        })
    };

    let set_filter = move |f: crate::store::FilterStatus| {
        store.filter.set(f);
    };

    let is_active = move |f: crate::store::FilterStatus| store.filter.get() == f;

    let username = move || {
        store.user.get().unwrap_or_else(|| "User".to_string())
    };

    let first_letter = move || {
        username().chars().next().unwrap_or('?').to_uppercase().to_string()
    };

    let on_push_toggle = move |checked: bool| {
        spawn_local(async move {
            if checked {
                crate::store::subscribe_to_push_notifications().await;
            } else {
                crate::store::unsubscribe_from_push_notifications().await;
            }
            if let Ok(enabled) = crate::store::is_push_subscribed().await {
                store.push_enabled.set(enabled);
            }
        });
    };

    view! {
        <SidenavHeader>
            <div class="flex items-center gap-2 px-2 py-4">
                <div class="flex size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground shadow-sm">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15.362 5.214A8.252 8.252 0 0112 21 8.25 8.25 0 016.038 7.048 8.287 8.287 0 009 9.6a8.983 8.983 0 013.361-6.867 8.21 8.25 0 003 2.48z" />
                    </svg>
                </div>
                <div class="grid flex-1 text-left text-sm leading-tight overflow-hidden">
                    <span class="truncate font-semibold text-foreground text-base">"VibeTorrent"</span>
                    <span class="truncate text-[10px] text-muted-foreground opacity-70">"v3.0.0"</span>
                </div>
            </div>
        </SidenavHeader>

        <SidenavContent>
            <SidenavGroup>
                <SidenavGroupLabel>"Filtreler"</SidenavGroupLabel>
                <SidenavGroupContent>
                    <SidenavMenu>
                        <SidebarItem 
                            active=Signal::derive(move || is_active(crate::store::FilterStatus::All))
                            on_click=move |_| set_filter(crate::store::FilterStatus::All)
                            icon="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
                            label="Tümü"
                            count=Signal::derive(total_count)
                        />
                        <SidebarItem 
                            active=Signal::derive(move || is_active(crate::store::FilterStatus::Downloading))
                            on_click=move |_| set_filter(crate::store::FilterStatus::Downloading)
                            icon="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3"
                            label="İndirilenler"
                            count=Signal::derive(downloading_count)
                        />
                        <SidebarItem 
                            active=Signal::derive(move || is_active(crate::store::FilterStatus::Seeding))
                            on_click=move |_| set_filter(crate::store::FilterStatus::Seeding)
                            icon="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5"
                            label="Gönderilenler"
                            count=Signal::derive(seeding_count)
                        />
                        <SidebarItem 
                            active=Signal::derive(move || is_active(crate::store::FilterStatus::Completed))
                            on_click=move |_| set_filter(crate::store::FilterStatus::Completed)
                            icon="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                            label="Tamamlananlar"
                            count=Signal::derive(completed_count)
                        />
                        <SidebarItem 
                            active=Signal::derive(move || is_active(crate::store::FilterStatus::Paused))
                            on_click=move |_| set_filter(crate::store::FilterStatus::Paused)
                            icon="M15.75 5.25v13.5m-7.5-13.5v13.5"
                            label="Durdurulanlar"
                            count=Signal::derive(paused_count)
                        />
                        <SidebarItem 
                            active=Signal::derive(move || is_active(crate::store::FilterStatus::Inactive))
                            on_click=move |_| set_filter(crate::store::FilterStatus::Inactive)
                            icon="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636"
                            label="Pasif"
                            count=Signal::derive(inactive_count)
                        />
                    </SidenavMenu>
                </SidenavGroupContent>
            </SidenavGroup>
        </SidenavContent>

        <SidenavFooter>
            <div class="flex flex-col gap-4 p-4">
                // Push Notification Toggle
                <div class="flex items-center justify-between px-2 py-1 bg-muted/20 rounded-md border border-border/50">
                    <div class="flex flex-col gap-0.5">
                        <span class="text-[10px] font-bold uppercase tracking-wider text-foreground/70">"Bildirimler"</span>
                        <span class="text-[9px] text-muted-foreground">"Web Push"</span>
                    </div>
                    <Switch 
                        checked=Signal::from(store.push_enabled) 
                        on_checked_change=Callback::new(on_push_toggle)
                    />
                </div>

                <div class="flex items-center gap-3 p-2 rounded-lg border bg-muted/30 shadow-xs overflow-hidden">
                    <div class="h-8 w-8 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-xs font-medium shrink-0 border border-primary-foreground/10">
                        {first_letter}
                    </div>
                    <div class="flex-1 overflow-hidden">
                        <div class="font-medium text-[11px] truncate text-foreground leading-tight">{username}</div>
                        <div class="text-[9px] text-muted-foreground truncate opacity-70">"Yönetici"</div>
                    </div>
                    
                    <div class="flex items-center gap-1">
                        <ThemeToggle />
                        
                        <Button
                            variant=ButtonVariant::Ghost
                            size=ButtonSize::Icon
                            class="size-7 text-destructive hover:bg-destructive/10"
                            on:click=move |_| {
                                spawn_local(async move {
                                    if shared::server_fns::auth::logout().await.is_ok() {
                                        let window = web_sys::window().expect("window should exist");
                                        let _ = window.location().set_href("/login");
                                    }
                                });
                            }
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-4">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15M12 9l-3 3m0 0l3 3m-3-3h12.75" />
                            </svg>
                        </Button>
                    </div>
                </div>
            </div>
        </SidenavFooter>
    }
}

#[component]
fn SidebarItem(
    active: Signal<bool>,
    on_click: impl Fn(web_sys::MouseEvent) + 'static + Send,
    #[prop(into)] icon: String,
    #[prop(into)] label: &'static str,
    count: Signal<usize>,
) -> impl IntoView {
    let variant = move || if active.get() { SidenavMenuButtonVariant::Outline } else { SidenavMenuButtonVariant::Default };
    let class = move || if active.get() { "bg-accent/50 border-accent text-foreground".to_string() } else { "text-muted-foreground hover:text-foreground".to_string() };

    view! {
        <SidenavMenuItem>
            <SidenavMenuButton
                variant=Signal::derive(variant)
                class=Signal::derive(class)
                on:click=on_click
            >
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-4 shrink-0">
                    <path stroke-linecap="round" stroke-linejoin="round" d=icon.clone() />
                </svg>
                <span class="flex-1 truncate">{label}</span>
                <span class="text-[10px] font-mono opacity-50">{count}</span>
            </SidenavMenuButton>
        </SidenavMenuItem>
    }
}
