use leptos::prelude::*;
use crate::components::torrent::add_torrent::AddTorrentDialog;

#[component]
pub fn Toolbar() -> impl IntoView {
    let show_add_modal = signal(false);
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let is_mobile_menu_open = use_context::<RwSignal<bool>>().expect("mobile menu state not provided");

    let search_value = RwSignal::new(String::new());
    
    // Sync search_value to store
    Effect::new(move |_| {
        let val = search_value.get();
        store.search_query.set(val);
    });

    view! {
        <div class="flex min-h-14 h-auto items-center border-b border-border bg-background px-4" style="padding-top: env(safe-area-inset-top);">
            // Sol kısım: Menü butonu + Add Torrent
            <div class="flex items-center gap-3">
                // Mobile Menu Trigger
                <button
                    class="inline-flex items-center justify-center size-9 rounded-md hover:bg-accent hover:text-accent-foreground lg:hidden"
                    on:click=move |_| is_mobile_menu_open.update(|v| *v = !*v)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="w-5 h-5 stroke-current"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
                </button>
                
                <button
                    class="inline-flex items-center justify-center gap-2 h-9 px-4 py-2 rounded-md text-sm font-medium bg-primary text-primary-foreground shadow-xs hover:bg-primary/90 transition-all active:scale-[0.98]"
                    on:click=move |_| show_add_modal.1.set(true)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4 md:w-5 md:h-5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                    </svg>
                    <span class="hidden sm:inline">"Add Torrent"</span>
                    <span class="sm:hidden">"Add"</span>
                </button>
            </div>

            // Sağ kısım: Search kutusu
            <div class="flex flex-1 items-center justify-end gap-2">
                <div class="hidden md:flex items-center gap-2 w-full max-w-xs">
                    <div class="relative flex-1">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground pointer-events-none">
                            <path stroke-linecap="round" stroke-linejoin="round" d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z" />
                        </svg>
                        <input
                            type="search"
                            placeholder="Search..."
                            class="file:text-foreground placeholder:text-muted-foreground border-input flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2 md:text-sm pl-8"
                            bind:value=search_value
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