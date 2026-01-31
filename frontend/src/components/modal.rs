use leptos::*;

#[component]
pub fn Modal(
    #[prop(into)] title: String,
    children: Children,
    #[prop(into)] on_confirm: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] visible: Signal<bool>,
    #[prop(into, default = "Confirm".to_string())] confirm_text: String,
    #[prop(into, default = "Cancel".to_string())] cancel_text: String,
    #[prop(into, default = false)] is_danger: bool,
) -> impl IntoView {
    let title = store_value(title);
    // Eagerly render children to a Fragment, which is Clone
    let child_view = store_value(children());
    let on_confirm = store_value(on_confirm);
    let on_cancel = store_value(on_cancel);
    let confirm_text = store_value(confirm_text);
    let cancel_text = store_value(cancel_text);
    
    view! {
        <Show when=move || visible.get() fallback=|| ()>
            <div class="fixed inset-0 bg-black/80 backdrop-blur-md flex items-end md:items-center justify-center z-[200] animate-in fade-in duration-200 sm:p-4">
                <div class="bg-[#16161c] p-6 rounded-t-2xl md:rounded-2xl w-full max-w-sm shadow-2xl border border-white/10 ring-1 ring-white/5 transform transition-all animate-in slide-in-from-bottom-10 md:slide-in-from-bottom-0 md:zoom-in-95">
                    <h3 class="text-xl font-bold text-white mb-4">{title.get_value()}</h3>
                    
                    <div class="text-gray-400 mb-8">
                        {child_view.with_value(|c| c.clone())}
                    </div>
                    
                    <div class="flex gap-3">
                        <button 
                            class="flex-1 px-4 py-3 bg-white/5 hover:bg-white/10 rounded-xl transition-all font-medium text-white"
                            on:click=move |_| on_cancel.with_value(|cb| cb.call(()))
                        >
                            {cancel_text.get_value()}
                        </button>
                        <button 
                            class=format!("flex-1 px-4 py-3 rounded-xl transition-all font-bold text-white shadow-lg {}", 
                                if is_danger { "bg-red-500 hover:bg-red-600 shadow-red-500/20" } else { "bg-blue-500 hover:bg-blue-600 shadow-blue-500/20" }
                            )
                            on:click=move |_| {
                                logging::log!("Modal: Confirm clicked");
                                on_confirm.with_value(|cb| cb.call(()))
                            }
                        >
                            {confirm_text.get_value()}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
