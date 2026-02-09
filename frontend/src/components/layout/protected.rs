use leptos::prelude::*;
use crate::components::layout::sidebar::Sidebar;
use crate::components::layout::toolbar::Toolbar;
use crate::components::layout::statusbar::StatusBar;

#[component]
pub fn Protected(children: Children) -> impl IntoView {
    view! {
        <div class="drawer lg:drawer-open h-full w-full">
            <input id="my-drawer" type="checkbox" class="drawer-toggle" />
            
            <div class="drawer-content flex flex-col h-full overflow-hidden bg-base-100">
                // --- TOOLBAR (TOP) ---
                <Toolbar />
                
                // --- MAIN CONTENT ---
                <main class="flex-1 overflow-hidden relative">
                    {children()}
                </main>

                // --- STATUS BAR (BOTTOM) ---
                <StatusBar />
            </div>

            // --- SIDEBAR (DRAWER) ---
            <div class="drawer-side z-[100]">
                <label for="my-drawer" aria-label="close sidebar" class="drawer-overlay"></label>
                <Sidebar />
            </div>
        </div>
    }
}