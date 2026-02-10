use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_shadcn_dialog::{Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter};
use leptos_shadcn_input::Input;
use leptos_shadcn_button::{Button, ButtonVariant};
use leptos_shadcn_alert::{Alert, AlertDescription, AlertVariant};
use crate::store::TorrentStore;
use crate::api;

#[component]
pub fn AddTorrentDialog(
    on_close: Callback<()>,
) -> impl IntoView {
    let store = use_context::<TorrentStore>().expect("TorrentStore not provided");
    let notifications = store.notifications;

    let uri = signal(String::new());
    let is_loading = signal(false);
    let error_msg = signal(Option::<String>::None);
    let is_open = signal(true);

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
                    crate::store::show_toast_with_signal(
                        notifications, 
                        shared::NotificationLevel::Success, 
                        "Torrent başarıyla eklendi"
                    );
                    is_open.1.set(false);
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


    view! {
        <Dialog
            open=Signal::derive(move || is_open.0.get())
            on_open_change=Callback::new(move |open: bool| {
                is_open.1.set(open);
                if !open {
                    on_close.run(());
                }
            })
        >
            <DialogContent class="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>"Add Torrent"</DialogTitle>
                    <DialogDescription>"Enter a Magnet link or a .torrent file URL."</DialogDescription>
                </DialogHeader>
                
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

                    <DialogFooter>
                        <Button
                            variant=ButtonVariant::Ghost
                            on_click=Callback::new(move |()| {
                                is_open.1.set(false);
                                on_close.run(());
                            })
                        >
                            "Cancel"
                        </Button>
                        <Button disabled=Signal::derive(move || is_loading.0.get())>
                            {move || if is_loading.0.get() {
                                leptos::either::Either::Left(view! { <span class="loading loading-spinner"></span> "Adding..." })
                            } else {
                                leptos::either::Either::Right(view! { "Add" })
                            }}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    }
}