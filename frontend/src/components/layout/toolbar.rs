use leptos::prelude::*;
use crate::components::torrent::add_torrent::AddTorrentDialog;

#[component]
pub fn Toolbar() -> impl IntoView {
    let show_add_modal = signal(false);
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    view! {
        <div class="navbar min-h-14 h-auto bg-base-100 p-0" style="padding-top: env(safe-area-inset-top);">
            <div class="navbar-start gap-4 px-4">
                <label for="my-drawer" class="btn btn-square btn-ghost lg:hidden drawer-button">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
                </label>
                
                <div class="flex items-center gap-3">
                    <button 
                        class="btn btn-primary btn-sm md:btn-md gap-2 shadow-md hover:shadow-primary/20 transition-all"
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

            <div class="navbar-center hidden md:flex">
                <div class="join shadow-sm border border-base-200">
                    <div class="relative">
                        <input 
                            type="text" 
                            placeholder="Search..." 
                            class="input input-sm input-bordered join-item w-full max-w-xs focus:outline-none" 
                            prop:value=move || store.search_query.get()
                            on:input=move |ev| store.search_query.set(event_target_value(&ev))
                        />
                        <Show when=move || !store.search_query.get().is_empty()>
                            <button 
                                class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs btn-circle"
                                on:click=move |_| store.search_query.set(String::new())
                            >
                                "×"
                            </button>
                        </Show>
                    </div>
                </div>
            </div>

            <div class="navbar-end px-4 gap-2">
                <Show when=move || show_add_modal.0.get()>
                    <AddTorrentDialog on_close=Callback::new(move |()| show_add_modal.1.set(false)) />
                </Show>
            </div>
        </div>
    }
}