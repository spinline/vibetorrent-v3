use leptos::prelude::*;
use crate::components::layout::sidebar::Sidebar;
use crate::components::layout::toolbar::Toolbar;
use crate::components::layout::statusbar::StatusBar;

#[component]
pub fn Protected(children: Children) -> impl IntoView {
    // Mobil menü durumu için bir sinyal oluşturuyoruz (RwSignal for easier passing)
    let is_mobile_menu_open = RwSignal::new(false);
    
    // Sinyali context olarak sağlıyoruz ki Toolbar ve Sidebar buna erişebilsin
    provide_context(is_mobile_menu_open);

    view! {
        <div class="flex h-screen w-full overflow-hidden bg-background">
            
            // --- SIDEBAR (Desktop: Sabit, Mobil: Overlay) ---
            <aside class=move || {
                let base = "fixed inset-y-0 left-0 z-50 w-64 transform transition-transform duration-300 ease-in-out border-r border-border bg-card lg:relative lg:translate-x-0";
                if is_mobile_menu_open.get() {
                    format!("{} translate-x-0", base)
                } else {
                    format!("{} -translate-x-full", base)
                }
            }>
                <Sidebar />
            </aside>

            // Mobil arka plan karartma (Overlay)
            <Show when=move || is_mobile_menu_open.get()>
                <div 
                    class="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm lg:hidden"
                    on:click=move |_| is_mobile_menu_open.set(false)
                ></div>
            </Show>
            
            // --- MAIN CONTENT AREA ---
            <div class="flex flex-1 flex-col overflow-hidden">
                // --- TOOLBAR (TOP) ---
                <Toolbar />
                
                // --- MAIN CONTENT ---
                <main class="flex-1 overflow-hidden relative bg-background">
                    {children()}
                </main>

                // --- STATUS BAR (BOTTOM) ---
                <StatusBar />
            </div>
        </div>
    }
}