use leptos::*;
use gloo_net::http::Request;
use crate::models::Torrent;

#[component]
pub fn ContextMenu(
    position: (i32, i32),
    visible: bool,
    torrent_hash: String,
    on_close: Callback<()>,
) -> impl IntoView {
    let handle_action = move |action: &str| {
        let hash = torrent_hash.clone();
        let action_str = action.to_string();
        let close = on_close.clone();
        
        spawn_local(async move {
            let body = serde_json::json!({
                "hash": hash,
                "action": action_str
            });
            
            let _ = Request::post("/api/torrents/action")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap() // Unwrap the Result<RequestBuilder, JsValue>
                .send()
                .await;
                
            close.call(());
        });
    };

    if !visible {
        return view! {}.into_view();
    }

    view! {
        <div 
            class="fixed inset-0 z-[100]" 
            on:click=move |_| on_close.call(())
            on:contextmenu=move |e| e.prevent_default()
        >
            <div 
                class="absolute bg-[#111116]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-2 min-w-[200px] animate-in fade-in zoom-in-95 duration-100"
                style=format!("left: {}px; top: {}px", position.0, position.1)
                on:click=move |e| e.stop_propagation()
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
        </div>
    }.into_view()
}
