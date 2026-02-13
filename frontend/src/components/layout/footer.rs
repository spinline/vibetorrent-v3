use leptos::prelude::*;
use crate::components::ui::separator::Separator;

#[component]
pub fn Footer() -> impl IntoView {
    let year = chrono::Local::now().format("%Y").to_string();
    
    view! {
        <footer class="mt-auto pb-6 px-4">
            <Separator class="mb-4 opacity-30" />
            <div class="flex items-center justify-center gap-2 text-[10px] uppercase tracking-widest text-muted-foreground/60 font-medium">
                <span>{format!("© {} VibeTorrent", year)}</span>
                <span class="size-1 rounded-full bg-muted-foreground/30" />
                <span>"v3.0.0-beta"</span>
            </div>
        </footer>
    }
}
