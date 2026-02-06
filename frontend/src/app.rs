use crate::components::layout::sidebar::Sidebar;
use crate::components::layout::statusbar::StatusBar;
use crate::components::layout::toolbar::Toolbar;
use crate::components::toast::ToastContainer;
use crate::components::torrent::table::TorrentTable;
use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    crate::store::provide_torrent_store();

    // Initialize push notifications after user grants permission
    create_effect(move |_| {
        spawn_local(async {
            // Wait a bit for service worker to be ready
            gloo_timers::future::TimeoutFuture::new(2000).await;
            
            // Check if running on iOS and not standalone
            if let Some(ios_message) = crate::utils::platform::get_ios_notification_info() {
                log::warn!("iOS detected: {}", ios_message);
                
                // Show toast to inform user
                if let Some(store) = use_context::<crate::store::TorrentStore>() {
                    crate::store::show_toast_with_signal(
                        store.notifications,
                        shared::NotificationLevel::Info,
                        ios_message,
                    );
                }
                return;
            }
            
            // Check if push notifications are supported
            if !crate::utils::platform::supports_push_notifications() {
                log::warn!("Push notifications not supported on this platform");
                return;
            }
            
            // Safari requires user gesture for notification permission
            // Don't auto-request on Safari - user should click a button
            if crate::utils::platform::is_safari() {
                log::info!("Safari detected - notification permission requires user interaction");
                if let Some(store) = use_context::<crate::store::TorrentStore>() {
                    crate::store::show_toast_with_signal(
                        store.notifications,
                        shared::NotificationLevel::Info,
                        "Bildirim izni için sağ alttaki ayarlar ⚙️ ikonuna basın.".to_string(),
                    );
                }
                return;
            }
            
            // For non-Safari browsers (Chrome, Firefox, Edge), attempt auto-subscribe
            log::info!("Attempting to subscribe to push notifications...");
            crate::store::subscribe_to_push_notifications().await;
        });
    });

    view! {
        // Main app wrapper - ensures proper stacking context
        <div class="relative w-full h-screen" style="height: 100dvh;">
            // Drawer layout
            <div class="drawer lg:drawer-open h-full w-full">
                <input id="my-drawer" type="checkbox" class="drawer-toggle" />

                <div class="drawer-content flex flex-col h-full overflow-x-hidden bg-base-100 text-base-content text-sm select-none">
                    // Toolbar at the top
                    <Toolbar />

                    <main class="flex-1 flex flex-col min-w-0 bg-base-100 overflow-hidden">
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
            
            // Toast container - fixed positioning relative to viewport
            <ToastContainer />
        </div>
    }
}
