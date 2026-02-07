use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[component]
pub fn Login() -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (loading, set_loading) = create_signal(false);

    let handle_login = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            let req = LoginRequest {
                username: username.get(),
                password: password.get(),
            };

            let client = gloo_net::http::Request::post("/api/auth/login")
                .json(&req)
                .expect("Failed to create request");

            match client.send().await {
                Ok(resp) => {
                    if resp.ok() {
                        // Redirect to home on success
                        let navigate = use_navigate();
                        navigate("/", Default::default());
                    } else {
                        set_error.set(Some("Kullanıcı adı veya şifre hatalı".to_string()));
                    }
                }
                Err(_) => {
                    set_error.set(Some("Bağlantı hatası".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="flex items-center justify-center min-h-screen bg-base-200">
            <div class="card w-full max-w-sm shadow-xl bg-base-100">
                <div class="card-body">
                    <h2 class="card-title justify-center mb-4">"VibeTorrent Giriş"</h2>

                    <form on:submit=handle_login>
                        <div class="form-control w-full">
                            <label class="label">
                                <span class="label-text">"Kullanıcı Adı"</span>
                            </label>
                            <input
                                type="text"
                                placeholder="Kullanıcı adınız"
                                class="input input-bordered w-full"
                                prop:value=username
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                disabled=move || loading.get()
                            />
                        </div>

                        <div class="form-control w-full mt-4">
                            <label class="label">
                                <span class="label-text">"Şifre"</span>
                            </label>
                            <input
                                type="password"
                                placeholder="******"
                                class="input input-bordered w-full"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                disabled=move || loading.get()
                            />
                        </div>

                        <Show when=move || error.get().is_some()>
                            <div class="alert alert-error mt-4 text-sm py-2">
                                <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                                <span>{move || error.get()}</span>
                            </div>
                        </Show>

                        <div class="card-actions justify-end mt-6">
                            <button
                                class="btn btn-primary w-full"
                                type="submit"
                                disabled=move || loading.get()
                            >
                                <Show when=move || loading.get() fallback=|| "Giriş Yap">
                                    <span class="loading loading-spinner"></span>
                                    "Giriş Yapılıyor..."
                                </Show>
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}
