use leptos::*;

#[component]
pub fn Toolbar() -> impl IntoView {
    let (show_add_modal, set_show_add_modal) = create_signal(false);

    view! {
        <div class="navbar min-h-14 h-14 bg-base-100 p-0">
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
                 <input type="text" placeholder="Filter..." class="input input-sm input-bordered w-full max-w-xs" />
            </div>

            <Show when=move || show_add_modal.get()>
                <crate::components::torrent::add_torrent::AddTorrentModal on_close=move |_| set_show_add_modal.set(false) />
            </Show>
        </div>
    }
}
