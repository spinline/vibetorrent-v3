use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::components::ui::input::{Input, InputType};
use crate::store::TorrentStore;
use crate::api;

use crate::components::ui::button::{Button, ButtonVariant};

#[component]
pub fn AddTorrentDialog(
    on_close: Callback<()>,
) -> impl IntoView {
    let _store = use_context::<TorrentStore>().expect("TorrentStore not provided");

    let uri = RwSignal::new(String::new());
    let is_loading = signal(false);
    let error_msg = signal(Option::<String>::None);

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let uri_val = uri.get();
        
        if uri_val.is_empty() {
            error_msg.1.set(Some("Please enter a Magnet URI or URL".to_string()));
            return;
        }

        is_loading.1.set(true);
        error_msg.1.set(None);

        let on_close = on_close.clone();
        spawn_local(async move {
            match api::torrent::add(&uri_val).await {
                Ok(_) => {
                    log::info!("Torrent added successfully");
                    crate::store::toast_success("Torrent başarıyla eklendi");
                    on_close.run(());
                }
                Err(e) => {
                    log::error!("Failed to add torrent: {:?}", e);
                    error_msg.1.set(Some(format!("Hata: {:?}", e)));
                    is_loading.1.set(false);
                }
            }
        });
    };

    let handle_backdrop = {
        let on_close = on_close.clone();
        move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            on_close.run(());
        }
    };

    view! {
        // Backdrop overlay
        <div
            class="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm"
            on:click=handle_backdrop
        />
        // Dialog panel
        <div class="fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border bg-card p-6 shadow-lg rounded-lg sm:max-w-[425px]">
            // Header
            <div class="flex flex-col space-y-1.5 text-center sm:text-left">
                <h2 class="text-lg font-semibold leading-none tracking-tight">"Add Torrent"</h2>
                <p class="text-sm text-muted-foreground">"Enter a Magnet link or a .torrent file URL."</p>
            </div>
            
            <form on:submit=handle_submit class="space-y-4">
                <Input
                    r#type=InputType::Text
                    placeholder="magnet:?xt=urn:btih:..."
                    bind_value=uri
                    disabled=is_loading.0.get()
                />
                
                {move || error_msg.0.get().map(|msg| view! {
                    <div class="rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
                        {msg}
                    </div>
                })}

                <div class="flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2">
                    <Button
                        variant=ButtonVariant::Ghost
                        attr:r#type="button"
                        on:click=move |_| on_close.run(())
                    >
                        "Cancel"
                    </Button>
                    <Button
                        attr:r#type="submit"
                        attr:disabled=move || is_loading.0.get()
                    >
                        {move || if is_loading.0.get() {
                            leptos::either::Either::Left(view! { 
                                <span class="animate-spin mr-2 h-4 w-4 border-2 border-current border-t-transparent rounded-full"></span> 
                                "Adding..." 
                            })
                        } else {
                            leptos::either::Either::Right(view! { "Add" })
                        }}
                    </Button>
                </div>
            </form>

            // Close button (X)
            <Button 
                variant=ButtonVariant::Ghost
                class="absolute right-2 top-2 size-8 p-0 opacity-70 hover:opacity-100"
                on:click=move |_| on_close.run(())
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-4 w-4">
                    <path d="M18 6 6 18"></path>
                    <path d="m6 6 12 12"></path>
                </svg>
                <span class="sr-only">"Close"</span>
            </Button>
        </div>
    }
}