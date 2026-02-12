use leptos::prelude::*;
use crate::components::ui::separator::Separator;

#[component]
pub fn Footer() -> impl IntoView {
    let year = chrono::Local::now().format("%Y").to_string();
    
    view! {
        <footer class="mt-auto px-4 py-6 md:px-8">
            <Separator class="mb-6 opacity-50" />
            <div class="flex flex-col items-center justify-between gap-4 md:flex-row">
                <p class="text-center text-sm leading-loose text-muted-foreground md:text-left">
                    {format!("© {} VibeTorrent. Tüm hakları saklıdır.", year)}
                </p>
                <div class="flex items-center gap-4 text-sm font-medium text-muted-foreground">
                    <a 
                        href="https://git.karatatar.com/admin/vibetorrent" 
                        target="_blank" 
                        rel="noreferrer" 
                        class="underline underline-offset-4 hover:text-foreground transition-colors"
                    >
                        "Gitea"
                    </a>
                    <span class="size-1 rounded-full bg-muted-foreground/30" />
                    <span class="text-[10px] tracking-widest uppercase opacity-70">"v3.0.0-beta"</span>
                </div>
            </div>
        </footer>
    }
}
