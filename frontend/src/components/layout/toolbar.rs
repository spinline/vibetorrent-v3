use leptos::*;

#[component]
pub fn Toolbar() -> impl IntoView {
    let (show_add_modal, set_show_add_modal) = create_signal(false);
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    view! {
        <div class="navbar min-h-14 h-auto bg-base-100 p-0" style="padding-top: env(safe-area-inset-top);">
            <div class="navbar-start gap-4 px-4">
                <label for="my-drawer" class="btn btn-square btn-ghost lg:hidden drawer-button">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
                </label>

                <div class="flex gap-2">
                    <button
                        class="btn btn-sm btn-primary gap-2 font-normal"
                        title="Add Magnet Link"
                        on:click=move |_| set_show_add_modal.set(true)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                        </svg>
                         "Add Torrent"
                    </button>
                </div>


            </div>

            <div class="navbar-end gap-2 px-4">
                 <div class="join">
                    <input
                        type="text"
                        placeholder="Search..."
                        class="input input-sm input-bordered join-item w-full max-w-xs focus:outline-none"
                        prop:value=move || store.search_query.get()
                        on:input=move |ev| store.search_query.set(event_target_value(&ev))
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Escape" {
                                store.search_query.set(String::new());
                            }
                        }
                    />
                    <Show when=move || !store.search_query.get().is_empty()>
                        <button
                            class="btn btn-sm btn-ghost join-item border-base-content/20 border-l-0 px-2"
                            title="Clear Search"
                            on:click=move |_| store.search_query.set(String::new())
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4 opacity-70">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </Show>
                 </div>
            </div>

            <Show when=move || show_add_modal.get()>
                <crate::components::torrent::add_torrent::AddTorrentModal on_close=move |_| set_show_add_modal.set(false) />
            </Show>
        </div>
    }
}
