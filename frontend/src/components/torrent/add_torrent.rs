use leptos::prelude::*;
use leptos::html;
use leptos::task::spawn_local;
use crate::store::TorrentStore;
use crate::api;

#[component]
pub fn AddTorrentDialog(
    on_close: Callback<()>,
) -> impl IntoView {
    let store = use_context::<TorrentStore>().expect("TorrentStore not provided");
    let notifications = store.notifications;

    let dialog_ref = NodeRef::<html::Dialog>::new();
    let uri = signal(String::new());
    let is_loading = signal(false);
    let error_msg = signal(Option::<String>::None);

    Effect::new(move |_| {
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.show_modal();
        }
    });

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
                    if let Some(dialog) = dialog_ref.get() {
                        dialog.close();
                    }
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

    let handle_cancel = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            dialog.close();
        }
        on_close.run(());
    };

    view! {
        <dialog node_ref=dialog_ref class="modal modal-bottom sm:modal-middle">
            <div class="modal-box">
                <h3 class="font-bold text-lg">"Add Torrent"</h3>
                <p class="py-4 text-sm opacity-70">"Enter a Magnet link or a .torrent file URL."</p>
                
                <form on:submit=handle_submit>
                    <div class="form-control w-full">
                        <input 
                            type="text" 
                            placeholder="magnet:?xt=urn:btih:..." 
                            class="input input-bordered w-full" 
                            prop:value=move || uri.0.get()
                            on:input=move |ev| uri.1.set(event_target_value(&ev))
                            disabled=move || is_loading.0.get()
                            autofocus
                        />
                    </div>
                    
                    <div class="modal-action">
                        <button type="button" class="btn btn-ghost" on:click=handle_cancel>"Cancel"</button>
                        <button type="submit" class="btn btn-primary" disabled=move || is_loading.0.get()>
                            {move || if is_loading.0.get() {
                                leptos::either::Either::Left(view! { <span class="loading loading-spinner"></span> "Adding..." })
                            } else {
                                leptos::either::Either::Right(view! { "Add" })
                            }}
                        </button>
                    </div>
                </form>

                {move || error_msg.0.get().map(|msg| view! {
                    <div class="text-error text-sm mt-2">{msg}</div>
                })}
            </div>
            <form method="dialog" class="modal-backdrop">
                <button on:click=handle_cancel>"close"</button>
            </form>
        </dialog>
    }
}