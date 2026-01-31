use leptos::*;
use leptos::html::Div;
use wasm_bindgen::JsCast;

pub fn use_click_outside(
    target: NodeRef<Div>,
    callback: impl Fn() + Clone + 'static,
) {
    create_effect(move |_| {
        if let Some(_) = target.get() {
            let handle_click = {
                let callback = callback.clone();
                let target = target.clone();
                move |ev: web_sys::MouseEvent| {
                    if let Some(el) = target.get() {
                        let ev_target = ev.target().unwrap().unchecked_into::<web_sys::Node>();
                        let el_node = el.unchecked_ref::<web_sys::Node>();
                        if !el_node.contains(Some(&ev_target)) {
                            callback();
                        }
                    }
                }
            };

            let window = web_sys::window().unwrap();
            let closure = wasm_bindgen::closure::Closure::<dyn FnMut(_)>::new(handle_click);
            let _ = window.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            
            // Cleanup
            on_cleanup(move || {
                let window = web_sys::window().unwrap();
                let _ = window.remove_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            });
        }
    });
}


#[component]
pub fn ContextMenu(
    position: (i32, i32),
    visible: bool,
    torrent_hash: String,
    on_close: Callback<()>,
    on_action: Callback<(String, String)>, // (Action, Hash)
) -> impl IntoView {
    let handle_action = move |action: &str| {
        let hash = torrent_hash.clone();
        let action_str = action.to_string();
        
        logging::log!("ContextMenu: Action '{}' for hash '{}'", action_str, hash);
        on_action.call((action_str, hash)); // Delegate FIRST
        on_close.call(()); // Close menu AFTER
    };

    let target = create_node_ref::<Div>();

    use_click_outside(target, move || {
        if visible {
            on_close.call(());
        }
    });

    if !visible {
        return view! {}.into_view();
    }

    view! {
        <div 
            node_ref=target
            class="fixed z-[100] bg-[#111116]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-2 min-w-[200px] animate-in fade-in zoom-in-95 duration-100"
            style=format!("left: {}px; top: {}px", position.0, position.1)
            on:contextmenu=move |e| e.prevent_default()
        >
            <div class="px-3 py-1 text-xs font-bold text-gray-500 uppercase tracking-wider mb-1">"Actions"</div>
            
            <button 
                class="w-full text-left px-4 py-2.5 hover:bg-white/10 flex items-center gap-3 transition-colors text-white"
                on:click={
                    let handle_action = handle_action.clone();
                    move |_| handle_action("start")
                }
            >
                <svg class="w-4 h-4 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                "Resume"
            </button>
            
            <button 
                class="w-full text-left px-4 py-2.5 hover:bg-white/10 flex items-center gap-3 transition-colors text-white"
                on:click={
                    let handle_action = handle_action.clone();
                    move |_| handle_action("stop")
                }
            >
                <svg class="w-4 h-4 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                "Pause"
            </button>
            
            <div class="h-px bg-white/10 my-1"></div>
            
            <button 
                class="w-full text-left px-4 py-2.5 hover:bg-red-500/20 text-red-500 hover:text-red-400 flex items-center gap-3 transition-colors"
                on:click={
                    let handle_action = handle_action.clone();
                    move |_| handle_action("delete")
                }
            >
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                "Delete"
            </button>
            
            <button 
                class="w-full text-left px-4 py-2.5 hover:bg-red-900/20 text-red-600 hover:text-red-400 flex items-center gap-3 transition-colors text-xs"
                on:click={
                    let handle_action = handle_action.clone();
                    move |_| handle_action("delete_with_data")
                }
            >
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
                <span>"Delete with Data"</span>
            </button>
        </div>
    }.into_view()
}
