use crate::components::layout::protected::Protected;
use crate::components::toast::ToastContainer;
use crate::components::torrent::table::TorrentTable;
use crate::components::auth::login::Login;
use crate::components::auth::setup::Setup;
use crate::api;
use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    crate::store::provide_torrent_store();

    let (is_loading, set_is_loading) = create_signal(true);
    let (is_authenticated, set_is_authenticated) = create_signal(false);

    create_effect(move |_| {
        spawn_local(async move {
            logging::log!("App initialization started...");

            let setup_res = api::setup::get_status().await;

            match setup_res {
                Ok(status) => {
                    if !status.completed {
                        logging::log!("Setup not completed, redirecting to /setup");
                        let navigate = use_navigate();
                        navigate("/setup", Default::default());
                        set_is_loading.set(false);
                        return;
                    }
                }
                Err(e) => logging::error!("Failed to get setup status: {:?}", e),
            }

            let auth_res = api::auth::check_auth().await;

            match auth_res {
                Ok(true) => {
                    logging::log!("Authenticated!");

                    if let Ok(user_info) = api::auth::get_user().await {
                        if let Some(store) = use_context::<crate::store::TorrentStore>() {
                            store.user.set(Some(user_info.username));
                        }
                    }

                    set_is_authenticated.set(true);

                    let pathname = window().location().pathname().unwrap_or_default();
                    if pathname == "/login" || pathname == "/setup" {
                        logging::log!("Already authenticated, redirecting to home");
                        let navigate = use_navigate();
                        navigate("/", Default::default());
                    }
                }
                Ok(false) => {
                    logging::log!("Not authenticated");

                    let pathname = window().location().pathname().unwrap_or_default();
                    if pathname != "/login" && pathname != "/setup" {
                        let navigate = use_navigate();
                        navigate("/login", Default::default());
                    }
                }
                Err(e) => {
                    logging::error!("Auth check failed: {:?}", e);
                    let navigate = use_navigate();
                    navigate("/login", Default::default());
                }
            }

            set_is_loading.set(false);
        });
    });

    create_effect(move |_| {
        if is_authenticated.get() {
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
