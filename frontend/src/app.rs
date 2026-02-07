use crate::components::layout::protected::Protected;
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

#[derive(Deserialize)]
struct UserResponse {
    username: String,
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
            logging::log!("App initialization started...");

            // 1. Check Setup Status
            let setup_res = gloo_net::http::Request::get("/api/setup/status").send().await;

            match setup_res {
                Ok(resp) => {
                    if resp.ok() {
                        match resp.json::<SetupStatus>().await {
                            Ok(status) => {
                                if !status.completed {
                                    logging::log!("Setup not completed, redirecting to /setup");
                                    let navigate = use_navigate();
                                    navigate("/setup", Default::default());
                                    set_is_loading.set(false);
                                    return;
                                }
                            }
                            Err(e) => logging::error!("Failed to parse setup status: {}", e),
                        }
                    }
                }
                Err(e) => logging::error!("Network error checking setup status: {}", e),
            }

            // 2. Check Auth Status
            let auth_res = gloo_net::http::Request::get("/api/auth/check").send().await;

                        match auth_res {
                            Ok(resp) => {
                                if resp.status() == 200 {
                                    logging::log!("Authenticated!");

                                    // Parse user info
                                    if let Ok(user_info) = resp.json::<UserResponse>().await {
                                        if let Some(store) = use_context::<crate::store::TorrentStore>() {
                                            store.user.set(Some(user_info.username));
                                        }
                                    }

                                    set_is_authenticated.set(true);

                                    // If user is already authenticated but on login/setup page, redirect to home
                                    let pathname = window().location().pathname().unwrap_or_default();
                                    if pathname == "/login" || pathname == "/setup" {
                                        logging::log!("Already authenticated, redirecting to home");
                                        let navigate = use_navigate();
                                        navigate("/", Default::default());
                                    }
                                } else {
                                    logging::log!("Not authenticated, redirecting to /login");
                                    let navigate = use_navigate();
                                    let pathname = window().location().pathname().unwrap_or_default();
                                    if pathname != "/login" && pathname != "/setup" {
                                         navigate("/login", Default::default());
                                    }
                                }
                            }
                            Err(e) => logging::error!("Network error checking auth status: {}", e),
                        }
            set_is_loading.set(false);
        });
    });
    // Initialize push notifications (Only if authenticated)
    create_effect(move |_| {
        if is_authenticated.get() {
            spawn_local(async {
                // ... (Push notification logic kept same, shortened for brevity in this replace)
                // Wait a bit for service worker to be ready
                gloo_timers::future::TimeoutFuture::new(2000).await;

                if crate::utils::platform::supports_push_notifications() && !crate::utils::platform::is_safari() {
                     crate::store::subscribe_to_push_notifications().await;
                }
            });
        }
    });

    view! {
        <div class="relative w-full h-screen" style="height: 100dvh;">
            <Router>
                <Routes>
                    <Route path="/login" view=move || view! { <Login /> } />
                    <Route path="/setup" view=move || view! { <Setup /> } />

                    <Route path="/" view=move || {
                        view! {
                            <Show when=move || !is_loading.get() fallback=|| view! {
                                <div class="flex items-center justify-center h-screen bg-base-100">
                                    <span class="loading loading-spinner loading-lg"></span>
                                </div>
                            }>
                                <Show when=move || is_authenticated.get() fallback=|| ()>
                                    <Protected>
                                        <TorrentTable />
                                    </Protected>
                                </Show>
                            </Show>
                        }
                    }/>

                    <Route path="/settings" view=move || {
                        view! {
                            <Show when=move || !is_loading.get() fallback=|| ()>
                                <Show when=move || is_authenticated.get() fallback=|| ()>
                                    <Protected>
                                        <div class="p-4">"Settings Page (Coming Soon)"</div>
                                    </Protected>
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
