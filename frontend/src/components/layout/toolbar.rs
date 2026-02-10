use leptos::prelude::*;
use crate::components::torrent::add_torrent::AddTorrentDialog;

#[component]
pub fn Toolbar() -> impl IntoView {
    let show_add_modal = signal(false);
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    view! {
        <div class="flex min-h-14 h-auto items-center border-b border-border bg-background px-4" style="padding-top: env(safe-area-inset-top);">
            <div class="flex flex-1 items-center gap-4">
                // Mobile Menu Trigger (Sheet Trigger in full impl)
                <button id="mobile-sheet-trigger" class="lg:hidden inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-10 w-10">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="w-5 h-5 stroke-current"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
                </button>
                
                <div class="flex items-center gap-3">
                    <button 
                        class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 shadow gap-2"
                        on:click=move |_| show_add_modal.1.set(true)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4 md:w-5 md:h-5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                        </svg>
                        <span class="hidden sm:inline">"Add Torrent"</span>
                        <span class="sm:hidden">"Add"</span>
                    </button>
                </div>
            </div>

            <div class="hidden md:flex items-center justify-center flex-1">
                <div class="relative w-full max-w-sm">
                    <input 
                        type="text" 
                        placeholder="Search..." 
                        class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50" 
                        prop:value=move || store.search_query.get()
                        on:input=move |ev| store.search_query.set(event_target_value(&ev))
                    />
                    <Show when=move || !store.search_query.get().is_empty()>
                        <button 
                            class="absolute right-2 top-1/2 -translate-y-1/2 inline-flex items-center justify-center rounded-full text-xs font-medium hover:bg-muted h-5 w-5 opacity-50 hover:opacity-100 transition-opacity"
                            on:click=move |_| store.search_query.set(String::new())
                        >
                            "×"
                        </button>
                    </Show>
                </div>
            </div>

            <div class="flex flex-1 justify-end px-4 gap-2">
                <Show when=move || show_add_modal.0.get()>
                    <AddTorrentDialog on_close=Callback::new(move |()| show_add_modal.1.set(false)) />
                </Show>
            </div>
        </div>
    }
}