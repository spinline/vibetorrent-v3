use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api;

#[component]
pub fn Setup() -> impl IntoView {
    let username = signal(String::new());
    let password = signal(String::new());
    let confirm_password = signal(String::new());
    let error = signal(Option::<String>::None);
    let loading = signal(false);

    let handle_setup = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        
        let pass = password.0.get();
        let confirm = confirm_password.0.get();
        
        if pass != confirm {
            error.1.set(Some("Şifreler eşleşmiyor".to_string()));
            return;
        }

        if pass.len() < 6 {
            error.1.set(Some("Şifre en az 6 karakter olmalıdır".to_string()));
            return;
        }

        loading.1.set(true);
        error.1.set(None);

        let user = username.0.get();

        spawn_local(async move {
            match api::setup::setup(&user, &pass).await {
                Ok(_) => {
                    log::info!("Setup completed successfully, redirecting...");
                    let window = web_sys::window().expect("window should exist");
                    let _ = window.location().set_href("/");
                }
                Err(e) => {
                    log::error!("Setup failed: {:?}", e);
                    error.1.set(Some(format!("Hata: {:?}", e)));
                    loading.1.set(false);
                }
            }
        });
    };

    view! {
        <div class="flex items-center justify-center min-h-screen bg-base-200">
            <div class="card w-full max-w-md shadow-xl bg-base-100">
                <div class="card-body">
                    <div class="flex flex-col items-center mb-6 text-center">
                        <div class="w-16 h-16 bg-primary rounded-2xl flex items-center justify-center text-primary-content shadow-lg mb-4">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-10 h-10">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M11.42 15.17L17.25 21A2.652 2.652 0 0021 17.25l-5.877-5.877M11.42 15.17l2.496-3.03c.317-.384.74-.626 1.208-.766M11.42 15.17l-4.655 5.653a2.548 2.548 0 11-3.586-3.586l6.837-5.63m5.108-3.497a2.548 2.548 0 113.586 3.586l-6.837 5.63m-5.108 3.497l2.496-3.03c.317-.384.74-.626 1.208-.766M15.75 9.25a2.548 2.548 0 11-5.096 0 2.548 2.548 0 015.096 0z" />
                            </svg>
                        </div>
                        <h2 class="card-title text-2xl font-bold">"VibeTorrent Kurulumu"</h2>
                        <p class="text-base-content/60 text-sm">"Yönetici hesabınızı oluşturun"</p>
                    </div>

                    <form on:submit=handle_setup class="space-y-4">
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text">"Yönetici Kullanıcı Adı"</span>
                            </label>
                            <input 
                                type="text" 
                                placeholder="admin" 
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
                            <label class="label">
                                <span class="label-text">"Şifre Onay"</span>
                            </label>
                            <input 
                                type="password" 
                                placeholder="******" 
                                class="input input-bordered w-full" 
                                prop:value=move || confirm_password.0.get()
                                on:input=move |ev| confirm_password.1.set(event_target_value(&ev))
                                disabled=move || loading.0.get()
                                required 
                            />
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
                                <Show when=move || loading.0.get() fallback=|| "Kurulumu Tamamla">
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