use crate::components::layout::protected::Protected;
use crate::components::toast::ToastContainer;
use crate::components::torrent::table::TorrentTable;
use crate::components::auth::login::Login;
use crate::components::auth::setup::Setup;
use crate::api;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::hooks::use_navigate;

#[component]
pub fn App() -> impl IntoView {
    crate::store::provide_torrent_store();

    let is_loading = signal(true);
    let is_authenticated = signal(false);

    Effect::new(move |_| {
        spawn_local(async move {
            log::info!("App initialization started...");

            let setup_res = api::setup::get_status().await;

            match setup_res {
                Ok(status) => {
                    if !status.completed {
                        log::info!("Setup not completed, redirecting to /setup");
                        let navigate = use_navigate();
                        navigate("/setup", Default::default());
                        is_loading.1.set(false);
                        return;
                    }
                }
                Err(e) => log::error!("Failed to get setup status: {:?}", e),
            }

            let auth_res = api::auth::check_auth().await;

            match auth_res {
                Ok(true) => {
                    log::info!("Authenticated!");

                    if let Ok(user_info) = api::auth::get_user().await {
                        if let Some(store) = use_context::<crate::store::TorrentStore>() {
                            store.user.set(Some(user_info.username));
                        }
                    }

                    is_authenticated.1.set(true);

                    let pathname = window().location().pathname().unwrap_or_default();
                    if pathname == "/login" || pathname == "/setup" {
                        log::info!("Already authenticated, redirecting to home");
                        let navigate = use_navigate();
                        navigate("/", Default::default());
                    }
                }
                Ok(false) => {
                    log::info!("Not authenticated");

                    let pathname = window().location().pathname().unwrap_or_default();
                    if pathname != "/login" && pathname != "/setup" {
                        let navigate = use_navigate();
                        navigate("/login", Default::default());
                    }
                }
                Err(e) => {
                    log::error!("Auth check failed: {:?}", e);
                    let navigate = use_navigate();
                    navigate("/login", Default::default());
                }
            }

            is_loading.1.set(false);
        });
    });

    Effect::new(move |_| {
        if is_authenticated.0.get() {
            spawn_local(async {
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
                <Routes fallback=|| view! { <div class="p-4">"404 Not Found"</div> }>
                    <Route path=leptos_router::path!("/login") view=move || view! { <Login /> } />
                    <Route path=leptos_router::path!("/setup") view=move || view! { <Setup /> } />

                    <Route path=leptos_router::path!("/") view=move || {
                        view! {
                            <Show when=move || !is_loading.0.get() fallback=|| view! {
                                <div class="flex items-center justify-center h-screen bg-base-100">
                                    <span class="loading loading-spinner loading-lg"></span>
                                </div>
                            }>
                                <Show when=move || is_authenticated.0.get() fallback=|| ()>
                                    <Protected>
                                        <TorrentTable />
                                    </Protected>
                                </Show>
                            </Show>
                        }
                    }/>

                    <Route path=leptos_router::path!("/settings") view=move || {
                        view! {
                            <Show when=move || !is_loading.0.get() fallback=|| ()>
                                <Show when=move || is_authenticated.0.get() fallback=|| ()>
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