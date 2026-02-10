use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_shadcn_button::{Button, ButtonVariant, ButtonSize};
use leptos_shadcn_avatar::{Avatar, AvatarFallback};
use leptos_shadcn_separator::Separator;

#[component]
pub fn Sidebar() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let is_mobile_menu_open = use_context::<RwSignal<bool>>().expect("mobile menu state not provided");

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
        is_mobile_menu_open.set(false);
    };

    let is_active = move |f: crate::store::FilterStatus| store.filter.get() == f;

    let username = move || {
        store.user.get().unwrap_or_else(|| "User".to_string())
    };

    let first_letter = move || {
        username().chars().next().unwrap_or('?').to_uppercase().to_string()
    };

    view! {
        <div class="w-full h-full flex flex-col bg-card" style="padding-top: env(safe-area-inset-top);">
            <div class="p-4 flex-1 overflow-y-auto">
                <div class="mb-4 px-2 text-lg font-semibold tracking-tight text-foreground">
                    "VibeTorrent"
                </div>
                <div class="space-y-1">
                    <h4 class="mb-1 rounded-md px-2 py-1 text-sm font-semibold text-muted-foreground">"Filters"</h4>
                    
                    <Button 
                        variant=MaybeProp::derive(move || Some(if is_active(crate::store::FilterStatus::All) { ButtonVariant::Secondary } else { ButtonVariant::Ghost }))
                        size=ButtonSize::Sm
                        class="w-full justify-start gap-2"
                        on_click=Callback::new(move |()| set_filter(crate::store::FilterStatus::All))
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                        </svg>
                        "All"
                        <span class="ml-auto text-xs font-mono opacity-70">{total_count}</span>
                    </Button>

                    <Button 
                        variant=MaybeProp::derive(move || Some(if is_active(crate::store::FilterStatus::Downloading) { ButtonVariant::Secondary } else { ButtonVariant::Ghost }))
                        size=ButtonSize::Sm
                        class="w-full justify-start gap-2"
                        on_click=Callback::new(move |()| set_filter(crate::store::FilterStatus::Downloading))
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
                        </svg>
                        "Downloading"
                        <span class="ml-auto text-xs font-mono opacity-70">{downloading_count}</span>
                    </Button>

                    <Button 
                        variant=MaybeProp::derive(move || Some(if is_active(crate::store::FilterStatus::Seeding) { ButtonVariant::Secondary } else { ButtonVariant::Ghost }))
                        size=ButtonSize::Sm
                        class="w-full justify-start gap-2"
                        on_click=Callback::new(move |()| set_filter(crate::store::FilterStatus::Seeding))
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
                        </svg>
                        "Seeding"
                        <span class="ml-auto text-xs font-mono opacity-70">{seeding_count}</span>
                    </Button>

                    <Button 
                        variant=MaybeProp::derive(move || Some(if is_active(crate::store::FilterStatus::Completed) { ButtonVariant::Secondary } else { ButtonVariant::Ghost }))
                        size=ButtonSize::Sm
                        class="w-full justify-start gap-2"
                        on_click=Callback::new(move |()| set_filter(crate::store::FilterStatus::Completed))
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        "Completed"
                        <span class="ml-auto text-xs font-mono opacity-70">{completed_count}</span>
                    </Button>

                    <Button 
                        variant=MaybeProp::derive(move || Some(if is_active(crate::store::FilterStatus::Paused) { ButtonVariant::Secondary } else { ButtonVariant::Ghost }))
                        size=ButtonSize::Sm
                        class="w-full justify-start gap-2"
                        on_click=Callback::new(move |()| set_filter(crate::store::FilterStatus::Paused))
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25v13.5m-7.5-13.5v13.5" />
                        </svg>
                        "Paused"
                        <span class="ml-auto text-xs font-mono opacity-70">{paused_count}</span>
                    </Button>

                    <Button 
                        variant=MaybeProp::derive(move || Some(if is_active(crate::store::FilterStatus::Inactive) { ButtonVariant::Secondary } else { ButtonVariant::Ghost }))
                        size=ButtonSize::Sm
                        class="w-full justify-start gap-2"
                        on_click=Callback::new(move |()| set_filter(crate::store::FilterStatus::Inactive))
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                        </svg>
                        "Inactive"
                         <span class="ml-auto text-xs font-mono opacity-70">{inactive_count}</span>
                    </Button>
                </div>
            </div>

            <Separator />

            <div class="p-4 bg-card">
                <div class="flex items-center gap-3">
                    <Avatar class="h-8 w-8">
                        <AvatarFallback class="bg-primary text-primary-foreground text-xs font-medium">
                            {first_letter}
                        </AvatarFallback>
                    </Avatar>
                    <div class="flex-1 overflow-hidden">
                        <div class="font-medium text-sm truncate text-foreground">{username}</div>
                        <div class="text-[10px] text-muted-foreground truncate">"Online"</div>
                    </div>
                    <Button
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Icon
                        class="text-destructive h-8 w-8"
                        on_click=Callback::new(move |()| {
                            spawn_local(async move {
                                if shared::server_fns::auth::logout().await.is_ok() {
                                    let window = web_sys::window().expect("window should exist");
                                    let _ = window.location().set_href("/login");
                                }
                            });
                        })
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15M12 9l-3 3m0 0l3 3m-3-3h12.75" />
                        </svg>
                    </Button>
                </div>
            </div>
        </div>
    }
}
