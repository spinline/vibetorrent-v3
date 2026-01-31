use leptos::*;
use thaw::*;

#[component]
pub fn StatusBar() -> impl IntoView {
    view! {
        <div class="h-8 border-t border-border bg-card/30 flex items-center px-4 text-xs space-x-4">
            <div class="flex items-center gap-1">
                <span class="i-mdi-arrow-down text-green-500"></span>
                "0 KB/s"
            </div>
            <div class="flex items-center gap-1">
                <span class="i-mdi-arrow-up text-blue-500"></span>
                "0 KB/s"
            </div>
            <div class="flex-1"></div>
            <div>"Free Space: 700 GB"</div>
        </div>
    }
}
