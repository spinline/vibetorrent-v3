use leptos::*;
use thaw::*;

#[component]
pub fn Toolbar(
    #[prop(into)] on_add: Callback<()>,
    #[prop(into)] on_start: Callback<()>,
    #[prop(into)] on_pause: Callback<()>,
    #[prop(into)] on_delete: Callback<()>,
    #[prop(into)] on_settings: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 p-2 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <Button variant=ButtonVariant::Text on_click=move |_| on_add.call(())>
                <span class="i-mdi-plus mr-2"/> "Add"
            </Button>
            <div class="h-4 w-px bg-border mx-2"></div>
            <Button variant=ButtonVariant::Text on_click=move |_| on_start.call(())>
                <span class="i-mdi-play mr-2"/> "Start"
            </Button>
            <Button variant=ButtonVariant::Text on_click=move |_| on_pause.call(())>
                <span class="i-mdi-pause mr-2"/> "Pause"
            </Button>
            <Button variant=ButtonVariant::Text color=ButtonColor::Error on_click=move |_| on_delete.call(())>
                <span class="i-mdi-delete mr-2"/> "Delete"
            </Button>
            <div class="flex-1"></div>
            <Input placeholder="Filter..." class="w-48" />
            <Button variant=ButtonVariant::Text on_click=move |_| on_settings.call(())>
                 <span class="i-mdi-cog mr-2"/> "Settings"
            </Button>
        </div>
    }
}
