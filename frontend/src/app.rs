use crate::components::layout::protected::Protected;
use crate::components::ui::skeleton::Skeleton;
use crate::components::ui::card::{Card, CardHeader, CardContent};
use crate::components::torrent::table::TorrentTable;
use crate::components::auth::login::Login;
use crate::components::auth::setup::Setup;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::hooks::{use_navigate, use_location};
use crate::components::ui::toast::Toaster;
use crate::components::hooks::use_theme_mode::ThemeMode;

#[component]
pub fn App() -> impl IntoView {
    crate::components::ui::toast::provide_toaster();
    let theme_mode = ThemeMode::init();

    // Sync theme with document
    Effect::new(move |_| {
        let is_dark = theme_mode.get();
        if let Some(doc) = document().document_element() {
            if is_dark {
                let _ = doc.class_list().add_1("dark");
                let _ = doc.set_attribute("data-theme", "dark");
            } else {
                let _ = doc.class_list().remove_1("dark");
                let _ = doc.set_attribute("data-theme", "light");
            }
        }
    });

    view! {
        <Toaster />
        <Router>
            <InnerApp />
        </Router>
    }
}

#[component]
fn InnerApp() -> impl IntoView {
    crate::store::provide_torrent_store();
    let store = use_context::<crate::store::TorrentStore>();
    let loc = use_location();

    let is_loading = signal(true);
    let is_authenticated = signal(false);
    let needs_setup = signal(false);

    Effect::new(move |_| {
        spawn_local(async move {
            log::info!("App initialization started...");
            gloo_console::log!("APP INIT: Checking setup status...");

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
            crate::store::toast_success("VibeTorrent'e Hoşgeldiniz");
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
                        <Show when=move || !is_loading.0.get() fallback=move || {
                            let path = loc.pathname.get();
                            if path == "/login" {
                                // Login Skeleton
                                view! {
                                    <div class="flex items-center justify-center min-h-screen bg-muted/40 px-4">
                                        <Card class="w-full max-w-sm shadow-lg border-none">
                                            <CardHeader class="pb-2 items-center space-y-4">
                                                <Skeleton class="w-12 h-12 rounded-xl" />
                                                <Skeleton class="h-8 w-32" />
                                                <Skeleton class="h-4 w-48" />
                                            </CardHeader>
                                            <CardContent class="pt-4 space-y-6">
                                                <div class="space-y-2">
                                                    <Skeleton class="h-4 w-24" />
                                                    <Skeleton class="h-10 w-full" />
                                                </div>
                                                <div class="space-y-2">
                                                    <Skeleton class="h-4 w-24" />
                                                    <Skeleton class="h-10 w-full" />
                                                </div>
                                                <Skeleton class="h-10 w-full rounded-md mt-4" />
                                            </CardContent>
                                        </Card>
                                    </div>
                                }.into_any()
                            } else {
                                // Dashboard Skeleton
                                view! {
                                    <div class="flex h-screen bg-background">
                                        // Sidebar skeleton
                                        <div class="w-56 border-r border-border p-4 space-y-4">
                                            <Skeleton class="h-8 w-3/4" />
                                            <div class="space-y-2">
                                                <Skeleton class="h-6 w-full" />
                                                <Skeleton class="h-6 w-full" />
                                                <Skeleton class="h-6 w-4/5" />
                                                <Skeleton class="h-6 w-full" />
                                                <Skeleton class="h-6 w-3/5" />
                                                <Skeleton class="h-6 w-full" />
                                            </div>
                                        </div>
                                        // Main content skeleton
                                        <div class="flex-1 flex flex-col">
                                            <div class="border-b border-border p-4 flex items-center gap-4">
                                                <Skeleton class="h-8 w-48" />
                                                <Skeleton class="h-8 w-64" />
                                                <div class="ml-auto"><Skeleton class="h-8 w-24" /></div>
                                            </div>
                                            <div class="flex-1 p-4 space-y-3">
                                                <Skeleton class="h-10 w-full" />
                                                <Skeleton class="h-10 w-full" />
                                                <Skeleton class="h-10 w-full" />
                                                <Skeleton class="h-10 w-full" />
                                                <Skeleton class="h-10 w-full" />
                                                <Skeleton class="h-10 w-3/4" />
                                            </div>
                                            <div class="border-t border-border p-3">
                                                <Skeleton class="h-5 w-96" />
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }>
                            <Show when=move || is_authenticated.0.get() fallback=|| ()>
                                <Protected>
                                    <div class="flex flex-col h-full overflow-hidden">
                                        <div class="flex-1 overflow-hidden">
                                            <TorrentTable />
                                        </div>
                                    </div>
                                </Protected>
                            </Show>
                        </Show>
                    }.into_any()
                }/>

                <Route path=leptos_router::path!("/settings") view=move || {
                    let authenticated = is_authenticated.0.get();
                    Effect::new(move |_| {
                        if !authenticated {
                            let navigate = use_navigate();
                            navigate("/login", Default::default());
                        }
                    });
                    
                    view! {
                        <Show when=move || !is_loading.0.get() fallback=|| ()>
                            <Show when=move || authenticated fallback=|| ()>
                                <Protected>
                                    <div class="p-4">"Settings Page (Coming Soon)"</div>
                                </Protected>
                            </Show>
                        </Show>
                    }
                }/>
            </Routes>
        </div>
    }
}
