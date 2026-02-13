use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::components::ui::sidenav::*;
use crate::components::ui::button::{Button, ButtonVariant, ButtonSize};
use crate::components::ui::theme_toggle::ThemeToggle;
use crate::components::ui::switch::Switch;

#[component]
pub fn Sidebar() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    
    // ... (existing counts and logic)
    
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
            // ... (VibeTorrent Header)
        </SidenavHeader>

        <SidenavContent>
            // ... (Filters)
        </SidenavContent>

        <SidenavFooter class="p-4 space-y-4">
            // Push Notification Toggle
            <div class="flex items-center justify-between px-2 py-1 bg-muted/20 rounded-md border border-border/50">
                <div class="flex flex-col gap-0.5">
                    <span class="text-[10px] font-bold uppercase tracking-wider text-foreground/70">"Bildirimler"</span>
                    <span class="text-[9px] text-muted-foreground">"Web Push"</span>
                </div>
                <Switch 
                    checked=store.push_enabled.into() 
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
        </SidenavFooter>
    }
}
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
