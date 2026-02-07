use leptos::*;
use crate::components::layout::sidebar::Sidebar;
use crate::components::layout::statusbar::StatusBar;
use crate::components::layout::toolbar::Toolbar;

#[component]
pub fn Protected(children: Children) -> impl IntoView {
    view! {
        <div class="drawer lg:drawer-open h-full w-full">
            <input id="my-drawer" type="checkbox" class="drawer-toggle" />

            <div class="drawer-content flex flex-col h-full overflow-hidden bg-base-100 text-base-content text-sm select-none">
                <Toolbar />

                <main class="flex-1 flex flex-col min-w-0 bg-base-100 overflow-hidden pb-8">
                    {children()}
                </main>

                <StatusBar />
            </div>

            <div class="drawer-side z-40 transition-none duration-0">
                <label for="my-drawer" aria-label="close sidebar" class="drawer-overlay transition-none duration-0"></label>
                <div class="menu p-0 min-h-full bg-base-200 text-base-content border-r border-base-300 transition-none duration-0">
                    <Sidebar />
                </div>
            </div>
        </div>
    }
}
