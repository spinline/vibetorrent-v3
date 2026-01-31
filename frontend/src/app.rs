use leptos::*;
use crate::components::layout::sidebar::Sidebar;
use crate::components::layout::toolbar::Toolbar;
use crate::components::layout::statusbar::StatusBar;
use crate::components::torrent::table::TorrentTable;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="flex flex-col h-screen w-screen overflow-hidden bg-base-100 text-base-content text-sm select-none">
            // Toolbar at the top
            <Toolbar />

            <div class="flex flex-1 overflow-hidden">
                // Sidebar on the left
                <Sidebar />

                // Main Content Area
                <main class="flex-1 flex flex-col min-w-0 bg-base-100">
                    <TorrentTable />
                </main>
            </div>

            // Status Bar at the bottom
            <StatusBar />
        </div>
    }
}
