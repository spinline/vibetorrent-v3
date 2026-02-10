use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_shadcn_button::{Button, ButtonVariant, ButtonSize};
use leptos_shadcn_avatar::{Avatar, AvatarFallback};
use leptos_shadcn_separator::Separator;

use leptos_use::storage::use_local_storage;
use ::codee::string::FromToStringCodec;

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

    // --- THEME LOGIC START ---
    let (current_theme, set_current_theme, _) = use_local_storage::<String, FromToStringCodec>("vibetorrent_theme");

    // Initialize with default if empty
    let current_theme_val = current_theme.get();
    if current_theme_val.is_empty() {
        set_current_theme.set("dark".to_string());
    }

    // Automatically sync theme to document attribute
    Effect::new(move |_| {
        let theme = current_theme.get().to_lowercase();
        if let Some(doc) = document().document_element() {
            let _ = doc.set_attribute("data-theme", &theme);
            // Also set class for Shadcn dark mode support
            if theme == "dark" || theme == "dracula" || theme == "dim" || theme == "abyss" || theme == "sunset" || theme == "cyberpunk" || theme == "nord" || theme == "business" || theme == "night" || theme == "black" || theme == "luxury" || theme == "coffee" || theme == "forest" || theme == "halloween" || theme == "synthwave" {
                let _ = doc.class_list().add_1("dark");
            } else {
                let _ = doc.class_list().remove_1("dark");
            }
        }
    });



    let toggle_theme = move |_| {
        let new_theme = if current_theme.get() == "dark" { "light" } else { "dark" };
        set_current_theme.set(new_theme.to_string());
    };
    // --- THEME LOGIC END ---

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

            <div class="p-4 bg-card" style="padding-bottom: calc(1rem + env(safe-area-inset-bottom));">
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

                    // --- THEME BUTTON ---
                    <Button
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Icon
                        class="h-8 w-8 text-muted-foreground hover:text-foreground"
                        on_click=Callback::new(toggle_theme)
                    >
                        // Sun icon for dark mode (to switch to light), Moon for light (to switch to dark)
                        // Actually show current state or action? Usually action.
                        // If dark, show Sun. If light, show Moon.
                        <Show when=move || current_theme.get() == "dark" fallback=|| view! {
                             <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M21.752 15.002A9.718 9.718 0 0118 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 003 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 009.002-5.998z" />
                            </svg>
                        }>
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z" />
                            </svg>
                        </Show>
                    </Button>
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
