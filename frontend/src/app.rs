use crate::components::layout::sidebar::Sidebar;
use crate::components::layout::statusbar::StatusBar;
use crate::components::layout::toolbar::Toolbar;
use crate::components::toast::ToastContainer;
use crate::components::torrent::table::TorrentTable;
use crate::components::auth::login::Login;
use crate::components::auth::setup::Setup;
use leptos::*;
use leptos_router::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct SetupStatus {
    completed: bool,
}

#[component]
pub fn App() -> impl IntoView {
    crate::store::provide_torrent_store();

    // Auth State
    let (is_loading, set_is_loading) = create_signal(true);
    let (is_authenticated, set_is_authenticated) = create_signal(false);

    // Check Auth & Setup Status on load
    create_effect(move |_| {
        spawn_local(async move {
            // 1. Check Setup Status
            let setup_res = gloo_net::http::Request::get("/api/setup/status").send().await;
            if let Ok(resp) = setup_res {
                if let Ok(status) = resp.json::<SetupStatus>().await {
                    if !status.completed {
                        // Redirect to setup if not completed
                        let navigate = use_navigate();
                        navigate("/setup", Default::default());
                        set_is_loading.set(false);
                        return;
                    }
                }
            }

            // 2. Check Auth Status
            let auth_res = gloo_net::http::Request::get("/api/auth/check").send().await;
            if let Ok(resp) = auth_res {
                if resp.status() == 200 {
                    set_is_authenticated.set(true);

                    // Initialize push notifications logic only if authenticated
                    // ... (Push notification logic moved here or kept global but guarded)
                } else {
                    let navigate = use_navigate();
                    // If we are already on login or setup, don't redirect loop
                    let pathname = window().location().pathname().unwrap_or_default();
                    if pathname != "/login" && pathname != "/setup" {
                         navigate("/login", Default::default());
                    }
                }
            }
            set_is_loading.set(false);
        });
    });

    // Initialize push notifications after user grants permission (Only if authenticated)
    create_effect(move |_| {
        if is_authenticated.get() {
            spawn_local(async {
                // Wait a bit for service worker to be ready
                gloo_timers::future::TimeoutFuture::new(2000).await;

                // Check if running on iOS and not standalone
                if let Some(ios_message) = crate::utils::platform::get_ios_notification_info() {
                    log::warn!("iOS detected: {}", ios_message);
                    if let Some(store) = use_context::<crate::store::TorrentStore>() {
                        crate::store::show_toast_with_signal(
                            store.notifications,
                            shared::NotificationLevel::Info,
                            ios_message,
                        );
                    }
                    return;
                }

                if !crate::utils::platform::supports_push_notifications() {
                    return;
                }

                if crate::utils::platform::is_safari() {
                    if let Some(store) = use_context::<crate::store::TorrentStore>() {
                        crate::store::show_toast_with_signal(
                            store.notifications,
                            shared::NotificationLevel::Info,
                            "Bildirim izni için sağ alttaki ayarlar ⚙️ ikonuna basın.".to_string(),
                        );
                    }
                    return;
                }

                crate::store::subscribe_to_push_notifications().await;
            });
        }
    });

    view! {
        <div class="relative w-full h-screen" style="height: 100dvh;">
            <Router>
                <Routes>
                    <Route path="/login" view=move || view! { <Login /> } />
                    <Route path="/setup" view=move || view! { <Setup /> } />

                    <Route path="/*" view=move || {
                        view! {
                            <Show when=move || !is_loading.get() fallback=|| view! {
                                <div class="flex items-center justify-center h-screen bg-base-100">
                                    <span class="loading loading-spinner loading-lg"></span>
                                </div>
                            }>
                                <Show when=move || is_authenticated.get() fallback=|| view! { <Login /> }>
                                    // Protected Layout
                                    <div class="drawer lg:drawer-open h-full w-full">
                                        <input id="my-drawer" type="checkbox" class="drawer-toggle" />

                                        <div class="drawer-content flex flex-col h-full overflow-hidden bg-base-100 text-base-content text-sm select-none">
                                            <Toolbar />

                                            <main class="flex-1 flex flex-col min-w-0 bg-base-100 overflow-hidden pb-8">
                                                <Routes>
                                                    <Route path="/" view=move || view! { <TorrentTable /> } />
                                                    <Route path="/settings" view=move || view! { <div class="p-4">"Settings Page (Coming Soon)"</div> } />
                                                </Routes>
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
                                </Show>
                            </Show>
                        }
                    }/>
                </Routes>
            </Router>

            <ToastContainer />
        </div>
    }
}
