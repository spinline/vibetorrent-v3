use leptos::*;
use shared::GlobalLimitRequest;

#[component]
pub fn GlobalLimitModal(
    #[prop(into)] visible: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let stats = store.global_stats;

    let (down_limit_kb, set_down_limit_kb) = create_signal(0i64);
    let (up_limit_kb, set_up_limit_kb) = create_signal(0i64);

    create_effect(move |_| {
        if visible.get() {
            let s = stats.get_untracked();
            set_down_limit_kb.set(s.down_limit.unwrap_or(0) / 1024);
            set_up_limit_kb.set(s.up_limit.unwrap_or(0) / 1024);
        }
    });

    let on_save = move |_| {
        let down_val = down_limit_kb.get() * 1024;
        let up_val = up_limit_kb.get() * 1024;

        spawn_local(async move {
            let req_body = GlobalLimitRequest {
                max_download_rate: Some(down_val),
                max_upload_rate: Some(up_val),
            };

            let client =
                gloo_net::http::Request::post("/api/settings/global-limits").json(&req_body);

            match client {
                Ok(req) => {
                    if let Err(e) = req.send().await {
                        logging::error!("Failed to save limits: {}", e);
                    } else {
                        logging::log!("Limits saved");
                        on_close.call(());
                    }
                }
                Err(e) => logging::error!("Failed to create request: {}", e),
            }
        });
    };

    view! {
        <crate::components::modal::Modal
            title="Global Speed Limits"
            visible=visible
            on_confirm=Callback::from(on_save)
            on_cancel=on_close
            confirm_text="Save"
        >
            <div class="space-y-4">
                <div class="space-y-2">
                    <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                        "Max Download Speed (KB/s)"
                    </label>
                    <input
                        type="number"
                        min="0"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                        prop:value=move || down_limit_kb.get()
                        on:input=move |ev| {
                            let val = event_target_value(&ev).parse().unwrap_or(0);
                            set_down_limit_kb.set(val);
                        }
                    />
                    <p class="text-[0.8rem] text-muted-foreground">"Set to 0 for unlimited."</p>
                </div>

                <div class="space-y-2">
                    <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                        "Max Upload Speed (KB/s)"
                    </label>
                    <input
                        type="number"
                        min="0"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                        prop:value=move || up_limit_kb.get()
                        on:input=move |ev| {
                            let val = event_target_value(&ev).parse().unwrap_or(0);
                            set_up_limit_kb.set(val);
                        }
                    />
                    <p class="text-[0.8rem] text-muted-foreground">"Set to 0 for unlimited."</p>
                </div>
            </div>
        </crate::components::modal::Modal>
    }
}
