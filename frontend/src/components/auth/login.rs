use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api;

#[component]
pub fn Login() -> impl IntoView {
    let username = signal(String::new());
    let password = signal(String::new());
    let remember_me = signal(false);
    let error = signal(Option::<String>::None);
    let loading = signal(false);

    let handle_login = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        loading.1.set(true);
        error.1.set(None);

        let user = username.0.get();
        let pass = password.0.get();
        let rem = remember_me.0.get();

        log::info!("Attempting login for user: {}", user);

        spawn_local(async move {
            match api::auth::login(&user, &pass, rem).await {
                Ok(_) => {
                    log::info!("Login successful, redirecting...");
                    let window = web_sys::window().expect("window should exist");
                    let _ = window.location().set_href("/");
                }
                Err(e) => {
                    log::error!("Login failed: {:?}", e);
                    error.1.set(Some("Geçersiz kullanıcı adı veya şifre".to_string()));
                    loading.1.set(false);
                }
            }
        });
    };

    view! {
        <div class="flex items-center justify-center min-h-screen bg-base-200">
            <div class="card w-full max-w-sm shadow-xl bg-base-100">
                <div class="card-body">
                    <div class="flex flex-col items-center mb-6">
                        <div class="w-16 h-16 bg-primary rounded-2xl flex items-center justify-center text-primary-content shadow-lg mb-4">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-10 h-10">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M15.362 5.214A8.252 8.252 0 0112 21 8.25 8.25 0 016.038 7.048 8.287 8.287 0 009 9.6a8.983 8.983 0 013.361-6.867 8.21 8.25 0 003 2.48z" />
                                <path stroke-linecap="round" stroke-linejoin="round" d="M12 18a3.75 3.75 0 00.495-7.467 5.99 5.99 0 00-1.925 3.546 5.974 5.974 0 01-2.133-1A3.75 3.75 0 0012 18z" />
                            </svg>
                        </div>
                        <h2 class="card-title text-2xl font-bold">"VibeTorrent"</h2>
                        <p class="text-base-content/60 text-sm">"Hesabınıza giriş yapın"</p>
                    </div>

                    <form on:submit=handle_login class="space-y-4">
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text">"Kullanıcı Adı"</span>
                            </label>
                            <input 
                                type="text" 
                                placeholder="Kullanıcı adınız" 
                                class="input input-bordered w-full" 
                                prop:value=move || username.0.get()
                                on:input=move |ev| username.1.set(event_target_value(&ev))
                                disabled=move || loading.0.get()
                                required 
                            />
                        </div>
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text">"Şifre"</span>
                            </label>
                            <input 
                                type="password" 
                                placeholder="******" 
                                class="input input-bordered w-full" 
                                prop:value=move || password.0.get()
                                on:input=move |ev| password.1.set(event_target_value(&ev))
                                disabled=move || loading.0.get()
                                required 
                            />
                        </div>

                        <div class="form-control">
                            <label class="label cursor-pointer justify-start gap-3">
                                <input 
                                    type="checkbox" 
                                    class="checkbox checkbox-primary checkbox-sm" 
                                    prop:checked=move || remember_me.0.get()
                                    on:change=move |ev| remember_me.1.set(event_target_checked(&ev))
                                />
                                <span class="label-text">"Beni hatırla"</span>
                            </label>
                        </div>

                        <Show when=move || error.0.get().is_some() fallback=|| ()>
                            <div class="alert alert-error text-xs py-2 shadow-sm">
                                <span>{move || error.0.get().unwrap_or_default()}</span>
                            </div>
                        </Show>

                        <div class="form-control mt-6">
                            <button 
                                class="btn btn-primary w-full" 
                                type="submit"
                                disabled=move || loading.0.get()
                            >
                                <Show when=move || loading.0.get() fallback=|| "Giriş Yap">
                                    <span class="loading loading-spinner"></span>
                                </Show>
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}