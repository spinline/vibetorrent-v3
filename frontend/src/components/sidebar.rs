use leptos::*;
use thaw::*;
use shared::TorrentStatus;

#[component]
pub fn Sidebar(
    #[prop(into)] active_filter: Signal<Option<TorrentStatus>>,
    #[prop(into)] on_filter_change: Callback<Option<TorrentStatus>>,
) -> impl IntoView {
    view! {
        <div class="w-64 border-r border-border bg-card/30 flex flex-col">
            <div class="p-4 font-bold text-lg">"Groups"</div>
            <div class="flex-1 overflow-y-auto p-2 space-y-1">
                <Button 
                    variant=if active_filter.get().is_none() { ButtonVariant::Primary } else { ButtonVariant::Text }
                    class="w-full justify-start text-left"
                    on_click=move |_| on_filter_change.call(None)
                >
                    "All"
                </Button>
                <Button 
                     variant=if active_filter.get() == Some(TorrentStatus::Downloading) { ButtonVariant::Primary } else { ButtonVariant::Text }
                    class="w-full justify-start text-left"
                    on_click=move |_| on_filter_change.call(Some(TorrentStatus::Downloading))
                >
                    "Downloading"
                </Button>
                <Button 
                     variant=if active_filter.get() == Some(TorrentStatus::Seeding) { ButtonVariant::Primary } else { ButtonVariant::Text }
                    class="w-full justify-start text-left"
                    on_click=move |_| on_filter_change.call(Some(TorrentStatus::Seeding))
                >
                    "Seeding"
                </Button>
                <Button 
                     variant=if active_filter.get() == Some(TorrentStatus::Paused) { ButtonVariant::Primary } else { ButtonVariant::Text }
                    class="w-full justify-start text-left"
                    on_click=move |_| on_filter_change.call(Some(TorrentStatus::Paused))
                >
                    "Paused"
                </Button>
                <Button 
                     variant=if active_filter.get() == Some(TorrentStatus::Error) { ButtonVariant::Primary } else { ButtonVariant::Text }
                    class="w-full justify-start text-left"
                    on_click=move |_| on_filter_change.call(Some(TorrentStatus::Error))
                >
                    "Errors"
                </Button>
            </div>
        </div>
    }
}
