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
    let store = use_context::<crate::store::TorrentStore>();

    let is_loading = signal(true);
    let is_authenticated = signal(false);
    let needs_setup = signal(false);

    Effect::new(move |_| {
        spawn_local(async move {
            log::info!("App initialization started...");

            // Check if setup is needed via Server Function
            match shared::server_fns::auth::get_setup_status().await {
                Ok(status) => {
                    if !status.completed {
                        log::info!("Setup not completed");
                        needs_setup.1.set(true);
                        is_loading.1.set(false);
                        return;
                    }
                }
                Err(e) => log::error!("Failed to get setup status: {:?}", e),
            }

            // Check authentication via GetUser Server Function
            match shared::server_fns::auth::get_user().await {
                Ok(Some(user_info)) => {
                    log::info!("Authenticated as {}", user_info.username);
                    if let Some(s) = store {
                        s.user.set(Some(user_info.username));
                    }
                    is_authenticated.1.set(true);
                }
                Ok(None) => {
                    log::info!("Not authenticated");
                }
                Err(e) => {
                    log::error!("Auth check failed: {:?}", e);
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
                    <Route path=leptos_router::path!("/login") view=move || {
                        let authenticated = is_authenticated.0.get();
                        let setup_needed = needs_setup.0.get();
                        
                        Effect::new(move |_| {
                            if setup_needed {
                                let navigate = use_navigate();
                                navigate("/setup", Default::default());
                            } else if authenticated {
                                log::info!("Already authenticated, redirecting to home");
                                let navigate = use_navigate();
                                navigate("/", Default::default());
                            }
                        });
                        
                        view! { <Login /> }
                    } />
                    <Route path=leptos_router::path!("/setup") view=move || {
                        Effect::new(move |_| {
                            if is_authenticated.0.get() {
                                let navigate = use_navigate();
                                navigate("/", Default::default());
                            }
                        });
                        
                        view! { <Setup /> }
                    } />

                    <Route path=leptos_router::path!("/") view=move || {
                        let navigate = use_navigate();
                        Effect::new(move |_| {
                            if !is_loading.0.get() {
                                if needs_setup.0.get() {
                                    log::info!("Setup not completed, redirecting to setup");
                                    navigate("/setup", Default::default());
                                } else if !is_authenticated.0.get() {
                                    log::info!("Not authenticated, redirecting to login");
                                    navigate("/login", Default::default());
                                }
                            }
                        });
                        
                        view! {
                            <Show when=move || !is_loading.0.get() fallback=|| view! {
                                <div class="flex items-center justify-center h-screen bg-background">
                                    <div class="flex flex-col items-center gap-4">
                                        <div class="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full"></div>
                                        <p class="text-sm text-muted-foreground">"Yükleniyor..."</p>
                                    </div>
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
                        Effect::new(move |_| {
                            if !is_authenticated.0.get() {
                                let navigate = use_navigate();
                                navigate("/login", Default::default());
                            }
                        });
                        
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
