use leptos::prelude::*;

#[component]
pub fn Modal(
    #[prop(into)] title: String,
    children: ChildrenFn,
    #[prop(into)] on_confirm: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] visible: Signal<bool>,
    #[prop(into, default = "Confirm".to_string())] confirm_text: String,
    #[prop(into, default = "Cancel".to_string())] cancel_text: String,
    #[prop(into, default = false)] is_danger: bool,
) -> impl IntoView {
    let title = StoredValue::new_local(title);
    let on_confirm = StoredValue::new_local(on_confirm);
    let on_cancel = StoredValue::new_local(on_cancel);
    let confirm_text = StoredValue::new_local(confirm_text);
    let cancel_text = StoredValue::new_local(cancel_text);
    
    view! {
        <Show when=move || visible.get() fallback=|| ()>
            <div class="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-end md:items-center justify-center z-[200] animate-in fade-in duration-200 sm:p-4">
                <div class="bg-card p-6 rounded-t-2xl md:rounded-lg w-full max-w-sm shadow-xl border border-border ring-0 transform transition-all animate-in slide-in-from-bottom-10 md:slide-in-from-bottom-0 md:zoom-in-95">
                    <h3 class="text-lg font-semibold text-card-foreground mb-4">{move || title.get_value()}</h3>
                    
                    <div class="text-muted-foreground mb-6 text-sm">
                        {children()}
                    </div>
                    
                    <div class="flex justify-end gap-3">
                        <button 
                            class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2"
                            on:click=move |_| on_cancel.with_value(|cb| cb.run(()))
                        >
                            {move || cancel_text.get_value()}
                        </button>
                        <button 
                            class=move || crate::utils::cn(format!("inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 h-10 px-4 py-2 {}", 
                                if is_danger { "bg-destructive text-destructive-foreground hover:bg-destructive/90" } 
                                else { "bg-primary text-primary-foreground hover:bg-primary/90" }
                            ))
                            on:click=move |_| {
                                log::info!("Modal: Confirm clicked");
                                on_confirm.with_value(|cb| cb.run(()))
                            }
                        >
                            {move || confirm_text.get_value()}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
