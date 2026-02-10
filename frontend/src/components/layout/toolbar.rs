use leptos::prelude::*;
use leptos_shadcn_input::Input;
use leptos_shadcn_button::{Button, ButtonVariant, ButtonSize};
use crate::components::torrent::add_torrent::AddTorrentDialog;

#[component]
pub fn Toolbar() -> impl IntoView {
    let show_add_modal = signal(false);
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let is_mobile_menu_open = use_context::<RwSignal<bool>>().expect("mobile menu state not provided");

    view! {
        <div class="flex min-h-14 h-auto items-center border-b border-border bg-background px-4" style="padding-top: env(safe-area-inset-top);">
            // Sol kısım: Menü butonu + Add Torrent
            <div class="flex items-center gap-3">
                // Mobile Menu Trigger
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Icon
                    class="lg:hidden"
                    on_click=Callback::new(move |()| is_mobile_menu_open.update(|v| *v = !*v))
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="w-5 h-5 stroke-current"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
                </Button>
                
                <Button
                    class="gap-2 shadow"
                    on_click=Callback::new(move |()| show_add_modal.1.set(true))
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4 md:w-5 md:h-5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                    </svg>
                    <span class="hidden sm:inline">"Add Torrent"</span>
                    <span class="sm:hidden">"Add"</span>
                </Button>
            </div>

            // Sağ kısım: Search kutusu
            <div class="flex flex-1 items-center justify-end gap-2">
                <div class="hidden md:flex items-center gap-2 w-full max-w-xs">
                    <div class="relative flex-1">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground pointer-events-none">
                            <path stroke-linecap="round" stroke-linejoin="round" d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z" />
                        </svg>
                        <Input
                            input_type="search"
                            placeholder="Search..."
                            value=MaybeProp::derive(move || Some(store.search_query.get()))
                            on_change=Callback::new(move |val: String| store.search_query.set(val))
                            class="pl-8 h-9"
                        />
                    </div>
                </div>
            </div>

            <Show when=move || show_add_modal.0.get()>
                <AddTorrentDialog on_close=Callback::new(move |()| show_add_modal.1.set(false)) />
            </Show>
        </div>
    }
}