use leptos::prelude::*;
use leptos::html;
use leptos_use::on_click_outside;

fn handle_action(
    hash: String,
    action: &str,
    on_action: Callback<(String, String)>,
    on_close: Callback<()>,
) {
    log::info!("ContextMenu: Action '{}' for hash '{}'", action, hash);
    on_action.run((action.to_string(), hash));
    on_close.run(());
}

#[component]
pub fn ContextMenu(
    position: (i32, i32),
    torrent_hash: String,
    on_close: Callback<()>,
    on_action: Callback<(String, String)>,
) -> impl IntoView {
    let container_ref = NodeRef::<html::Div>::new();
    
    let _ = on_click_outside(container_ref, move |_| on_close.run(()));

    let (x, y) = position;
    
    let hash1 = torrent_hash.clone();
    let hash2 = torrent_hash.clone();
    let hash3 = torrent_hash.clone();
    let hash4 = torrent_hash.clone();
    let hash5 = torrent_hash;

    view! {
        <div 
            node_ref=container_ref
            class="fixed z-[100] min-w-[200px] animate-in fade-in zoom-in-95 duration-100"
            style=format!("left: {}px; top: {}px;", x, y)
            on:contextmenu=move |e| e.prevent_default()
        >
            <ul class="menu bg-base-200 shadow-xl rounded-box border border-base-300 p-1 gap-0.5">
                <li>
                    <button class="flex items-center gap-3 px-3 py-2 hover:bg-primary hover:text-primary-content rounded-lg transition-colors" on:click=move |_| {
                        handle_action(hash1.clone(), "start", on_action.clone(), on_close.clone());
                    }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.348a1.125 1.125 0 010 1.971l-11.54 6.347a1.125 1.125 0 01-1.667-.985V5.653z" />
                        </svg>
                        <span>"Start"</span>
                    </button>
                </li>
                <li>
                    <button class="flex items-center gap-3 px-3 py-2 hover:bg-primary hover:text-primary-content rounded-lg transition-colors" on:click=move |_| {
                        handle_action(hash2.clone(), "stop", on_action.clone(), on_close.clone());
                    }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25v13.5m-7.5-13.5v13.5" />
                        </svg>
                        <span>"Stop"</span>
                    </button>
                </li>
                <li>
                    <button class="flex items-center gap-3 px-3 py-2 hover:bg-primary hover:text-primary-content rounded-lg transition-colors" on:click=move |_| {
                        handle_action(hash3.clone(), "recheck", on_action.clone(), on_close.clone());
                    }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
                        </svg>
                        <span>"Recheck"</span>
                    </button>
                </li>
                <div class="divider my-0.5 opacity-50"></div>
                <li>
                    <button class="flex items-center gap-3 px-3 py-2 text-error hover:bg-error hover:text-error-content rounded-lg transition-colors" on:click=move |_| {
                        handle_action(hash4.clone(), "delete", on_action.clone(), on_close.clone());
                    }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.164h-2.34c-1.18 0-2.09.984-2.09 2.164v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
                        </svg>
                        <span>"Remove"</span>
                    </button>
                </li>
                <li>
                    <button class="flex items-center gap-3 px-3 py-2 text-error hover:bg-error hover:text-error-content rounded-lg transition-colors" on:click=move |_| {
                        handle_action(hash5.clone(), "delete_with_data", on_action.clone(), on_close.clone());
                    }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5m6 4.125l2.25 2.25m0 0l2.25 2.25M12 13.875l2.25-2.25M12 13.875l-2.25-2.25M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
                        </svg>
                        <span>"Remove Data"</span>
                    </button>
                </li>
            </ul>
        </div>
    }
}
