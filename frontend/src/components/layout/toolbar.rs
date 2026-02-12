use leptos::prelude::*;
use icons::PanelLeft;
use crate::components::torrent::add_torrent::AddTorrentDialog;
use crate::components::ui::button::{Button, ButtonVariant, ButtonSize};
use crate::components::ui::sheet::{Sheet, SheetContent, SheetTrigger, SheetDirection};
use crate::components::layout::sidebar::Sidebar;

#[component]
pub fn Toolbar() -> impl IntoView {
    let show_add_modal = signal(false);

    view! {
        <div class="flex min-h-14 h-auto items-center border-b border-border bg-background px-4" style="padding-top: env(safe-area-inset-top);">
            // Sol kısım: Menü butonu (Mobil) + Add Torrent
            <div class="flex items-center gap-3">
                
                // --- MOBILE SHEET (SIDEBAR) ---
                <div class="lg:hidden">
                    <Sheet>
                        <SheetTrigger variant=ButtonVariant::Ghost size=ButtonSize::Icon class="size-9">
                            <PanelLeft class="size-5" />
                            <span class="hidden">"Menüyü Aç"</span>
                        </SheetTrigger>
                        <SheetContent 
                            direction=SheetDirection::Left 
                            class="p-0 w-[18rem] bg-card border-r border-border"
                            hide_close_button=true
                        >
                            <div class="flex flex-col h-full overflow-hidden">
                                <Sidebar />
                            </div>
                        </SheetContent>
                    </Sheet>
                </div>
                
                <Button
                    on:click=move |_| show_add_modal.1.set(true)
                    class="gap-2"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4 md:w-5 md:h-5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                    </svg>
                    <span class="hidden sm:inline">"Add Torrent"</span>
                    <span class="sm:hidden">"Add"</span>
                </Button>
            </div>

            // Sağ kısım boş
            <div class="flex flex-1 items-center justify-end gap-2">
            </div>

            <Show when=move || show_add_modal.0.get()>
                <AddTorrentDialog on_close=Callback::new(move |()| show_add_modal.1.set(false)) />
            </Show>
        </div>
    }
}