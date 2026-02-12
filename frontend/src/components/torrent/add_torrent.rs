use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use crate::components::ui::input::{Input, InputType};
use crate::api;
use crate::components::ui::button::Button;
use crate::components::ui::dialog::{
    DialogBody, DialogHeader, DialogTitle, DialogDescription, DialogFooter, DialogClose
};

#[component]
pub fn AddTorrentDialogContent() -> impl IntoView {
    let uri = RwSignal::new(String::new());
    let is_loading = signal(false);
    let error_msg = signal(Option::<String>::None);

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let uri_val = uri.get();
        
        if uri_val.is_empty() {
            error_msg.1.set(Some("Lütfen bir Magnet URI veya URL girin".to_string()));
            return;
        }

        is_loading.1.set(true);
        error_msg.1.set(None);

        spawn_local(async move {
            match api::torrent::add(&uri_val).await {
                Ok(_) => {
                    log::info!("Torrent added successfully");
                    crate::store::toast_success("Torrent başarıyla eklendi");
                    
                    // Programmatically close the dialog by triggering the close button
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        if let Some(el) = doc.get_element_by_id("add-torrent-dialog") {
                            if let Some(close_btn) = el.query_selector("[data-dialog-close]").ok().flatten() {
                                let _ = close_btn.dyn_into::<web_sys::HtmlElement>().map(|btn| btn.click());
                            }
                        }
                    }
                    
                    uri.set(String::new());
                    is_loading.1.set(false);
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
        <DialogBody>
            <DialogHeader>
                <DialogTitle>"Add Torrent"</DialogTitle>
                <DialogDescription>
                    "Enter a Magnet link or a .torrent file URL."
                </DialogDescription>
            </DialogHeader>
            
            <form on:submit=handle_submit class="space-y-4 pt-4">
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

                <DialogFooter class="pt-2">
                    <DialogClose>
                        "Cancel"
                    </DialogClose>
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
                </DialogFooter>
            </form>
        </DialogBody>
    }
}
