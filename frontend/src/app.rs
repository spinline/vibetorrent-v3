use leptos::*;
use leptos_router::*;
use crate::components::layout::sidebar::Sidebar;
use crate::components::layout::toolbar::Toolbar;
use crate::components::layout::statusbar::StatusBar;
use crate::components::torrent::table::TorrentTable;

#[component]
pub fn App() -> impl IntoView {
    crate::store::provide_torrent_store();

    view! {
        <div class="drawer lg:drawer-open">
            <input id="my-drawer" type="checkbox" class="drawer-toggle" />
            
            <div class="drawer-content flex flex-col h-screen overflow-hidden bg-base-100 text-base-content text-sm select-none">
                // Toolbar at the top
                <Toolbar />

                <main class="flex-1 flex flex-col min-w-0 bg-base-100 overflow-hidden space-y-6">
                    <Router>
                        <Routes>
                            <Route path="/" view=move || view! { <TorrentTable /> } />
                            <Route path="/settings" view=move || view! { <div class="p-4">"Settings Page (Coming Soon)"</div> } />
                        </Routes>
                    </Router>
                </main>
                
                 // Status Bar at the bottom
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
