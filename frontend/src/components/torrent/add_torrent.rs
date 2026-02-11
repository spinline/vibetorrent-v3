use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_shadcn_input::Input;
use leptos_shadcn_button::{Button, ButtonVariant};
use leptos_shadcn_alert::{Alert, AlertDescription, AlertVariant};
use crate::store::TorrentStore;
use crate::api;

#[component]
pub fn AddTorrentDialog(
    on_close: Callback<()>,
) -> impl IntoView {
    let _store = use_context::<TorrentStore>().expect("TorrentStore not provided");

    let uri = signal(String::new());
    let is_loading = signal(false);
    let error_msg = signal(Option::<String>::None);

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let uri_val = uri.0.get();
        
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
                    input_type="text"
                    placeholder="magnet:?xt=urn:btih:..."
                    value=MaybeProp::derive(move || Some(uri.0.get()))
                    on_change=Callback::new(move |val: String| uri.1.set(val))
                    disabled=Signal::derive(move || is_loading.0.get())
                />
                
                {move || error_msg.0.get().map(|msg| view! {
                    <Alert variant=AlertVariant::Destructive>
                        <AlertDescription>{msg}</AlertDescription>
                    </Alert>
                })}

                <div class="flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2">
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=Callback::new(move |()| {
                            on_close.run(());
                        })
                    >
                        "Cancel"
                    </Button>
                    <Button disabled=Signal::derive(move || is_loading.0.get())>
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
            <button 
                class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none"
                on:click=move |_| on_close.run(())
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-4 w-4">
                    <path d="M18 6 6 18"></path>
                    <path d="m6 6 12 12"></path>
                </svg>
                <span class="sr-only">"Close"</span>
            </button>
        </div>
    }
}