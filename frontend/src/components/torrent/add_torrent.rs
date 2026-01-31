use leptos::*;
use leptos::html::Dialog;


#[component]
pub fn AddTorrentModal(
    #[prop(into)]
    on_close: Callback<()>,
) -> impl IntoView {
    let dialog_ref = create_node_ref::<Dialog>();
    let (uri, set_uri) = create_signal(String::new());
    let (is_loading, set_loading) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal(Option::<String>::None);

    // Effect to open the dialog when the component mounts/renders
    create_effect(move |_| {
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.show_modal();
        }
    });

    let handle_submit = move |_| {
        let uri_val = uri.get();
        if uri_val.is_empty() {
            set_error_msg.set(Some("Please enter a Magnet URI or URL".to_string()));
            return;
        }

        set_loading.set(true);
        set_error_msg.set(None);

        spawn_local(async move {
            let req_body = serde_json::json!({
                "uri": uri_val
            });

            match gloo_net::http::Request::post("/api/torrents/add")
                .json(&req_body)
            {
                Ok(req) => {
                    match req.send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                logging::log!("Torrent added successfully");
                                set_loading.set(false);
                                if let Some(dialog) = dialog_ref.get() {
                                    dialog.close();
                                }
                                on_close.call(());
                            } else {
                                let status = resp.status();
                                let text = resp.text().await.unwrap_or_default();
                                logging::error!("Failed to add torrent: {} - {}", status, text);
                                set_error_msg.set(Some(format!("Error {}: {}", status, text)));
                                set_loading.set(false);
                            }
                        }
                        Err(e) => {
                            logging::error!("Network error: {}", e);
                            set_error_msg.set(Some(format!("Network Error: {}", e)));
                            set_loading.set(false);
                        }
                    }
                }
                Err(e) => {
                    logging::error!("Serialization error: {}", e);
                    set_error_msg.set(Some(format!("Request Error: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    };

    let handle_close = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            dialog.close();
        }
        on_close.call(());
    };

    view! {
        <dialog node_ref=dialog_ref class="modal modal-bottom sm:modal-middle" on:close=move |_| on_close.call(())>
            <div class="modal-box">
                <h3 class="font-bold text-lg">"Add Torrent"</h3>
                <p class="py-4">"Enter a Magnet URI or direct URL to a .torrent file."</p>
                
                <div class="form-control w-full">
                    <input 
                        type="text" 
                        placeholder="magnet:?xt=urn:btih:..." 
                        class="input input-bordered w-full" 
                        prop:value=uri
                        on:input=move |ev| set_uri.set(event_target_value(&ev))
                        disabled=is_loading
                    />
                </div>

                <div class="modal-action">
                    <button class="btn" on:click=handle_close disabled=is_loading>"Cancel"</button>
                    <button class="btn btn-primary" on:click=handle_submit disabled=is_loading>
                        {move || if is_loading.get() {
                            view! { <span class="loading loading-spinner"></span> "Adding..." }.into_view()
                        } else {
                            view! { "Add" }.into_view()
                        }}
                    </button>
                </div>
                
                {move || error_msg.get().map(|msg| view! {
                    <div class="text-error text-sm mt-2">{msg}</div>
                })}
            </div>
            <form method="dialog" class="modal-backdrop">
                <button type="button" on:click=handle_close>"close"</button>
            </form>
        </dialog>
    }
}
