use leptos::*;

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
        on_action.call((action_str, hash)); // Delegate to parent
    };

    if !visible {
        return view! {}.into_view();
    }

    view! {
        // Backdrop to catch clicks outside
        <div
            class="fixed inset-0 z-[99] cursor-default"
            role="button"
            tabindex="-1"
            on:click=move |_| on_close.call(())
            on:contextmenu=move |e| e.prevent_default()
        ></div>

        <div
            class="fixed z-[100] min-w-[200px] animate-in fade-in zoom-in-95 duration-100"
            style=format!("left: {}px; top: {}px", position.0, position.1)
            on:contextmenu=move |e| e.prevent_default()
        >
            <ul class="menu bg-base-200 text-base-content rounded-box shadow-xl border border-white/5 p-2 gap-1">


                <li>
                    <button
                        class="gap-3 active:bg-primary active:text-primary-content"
                        on:click={
                            let handle_action = handle_action.clone();
                            move |_| handle_action("start")
                        }
                    >
                        <svg class="w-4 h-4 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                        "Resume"
                    </button>
                </li>

                <li>
                    <button
                        class="gap-3 active:bg-primary active:text-primary-content"
                        on:click={
                            let handle_action = handle_action.clone();
                            move |_| handle_action("stop")
                        }
                    >
                        <svg class="w-4 h-4 text-warning" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                        "Pause"
                    </button>
                </li>

                <div class="divider my-0 h-px p-0 opacity-10"></div>

                <li>
                    <button
                        class="gap-3 text-error hover:bg-error/10 active:bg-error active:text-error-content"
                        on:click={
                            let handle_action = handle_action.clone();
                            move |_| handle_action("delete")
                        }
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                        "Delete"
                    </button>
                </li>

                <li>
                    <button
                        class="gap-3 text-error hover:bg-error/10 active:bg-error active:text-error-content text-xs"
                        on:click={
                            let handle_action = handle_action.clone();
                            move |_| handle_action("delete_with_data")
                        }
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
                        <span>"Delete with Data"</span>
                    </button>
                </li>
            </ul>
        </div>
    }.into_view()
}
