use leptos::prelude::*;
use icons::{PanelLeft, Plus};
use crate::components::torrent::add_torrent::AddTorrentDialogContent;
use crate::components::ui::button::{ButtonVariant, ButtonSize};
use crate::components::ui::sheet::{Sheet, SheetContent, SheetTrigger, SheetDirection};
use crate::components::ui::dialog::{Dialog, DialogContent, DialogTrigger};
use crate::components::layout::sidebar::Sidebar;

#[component]
pub fn Toolbar() -> impl IntoView {
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
                
                <Dialog>
                    <DialogTrigger 
                        variant=ButtonVariant::Default 
                        class="gap-2"
                    >
                        <Plus class="w-4 h-4 md:w-5 md:h-5" />
                        <span class="hidden sm:inline">"Add Torrent"</span>
                        <span class="sm:hidden">"Add"</span>
                    </DialogTrigger>
                    <DialogContent id="add-torrent-dialog">
                        <AddTorrentDialogContent />
                    </DialogContent>
                </Dialog>
            </div>

            // Sağ kısım boş
            <div class="flex flex-1 items-center justify-end gap-2">
            </div>
        </div>
    }
}
