use leptos::prelude::*;
use leptos::task::spawn_local;

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
            match shared::server_fns::auth::setup(user, pass).await {
                Ok(_) => {
                    log::info!("Setup completed successfully, redirecting...");
                    let window = web_sys::window().expect("window should exist");
                    let _ = window.location().set_href("/");
                }
                Err(e) => {
                    log::error!("Setup failed: {:?}", e);
                    error.1.set(Some("Kurulum sırasında bir hata oluştu".to_string()));
                    loading.1.set(false);
                }
            }
        });
    };

    view! {
        <div class="flex items-center justify-center min-h-screen bg-muted/40 px-4">
            <div class="w-full max-w-md rounded-xl border border-border bg-card text-card-foreground shadow-lg overflow-hidden">
                <div class="flex flex-col space-y-1.5 p-6 pb-2 items-center text-center">
                    <div class="w-12 h-12 bg-primary rounded-xl flex items-center justify-center text-primary-foreground shadow-sm mb-4">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M11.42 15.17L17.25 21A2.652 2.652 0 0021 17.25l-5.877-5.877M11.42 15.17l2.496-3.03c.317-.384.74-.626 1.208-.766M11.42 15.17l-4.655 5.653a2.548 2.548 0 11-3.586-3.586l6.837-5.63m5.108-3.497a2.548 2.548 0 113.586 3.586l-6.837 5.63m-5.108 3.497l2.496-3.03c.317-.384.74-.626 1.208-.766M15.75 9.25a2.548 2.548 0 11-5.096 0 2.548 2.548 0 015.096 0z" />
                        </svg>
                    </div>
                    <h3 class="font-semibold tracking-tight text-2xl">"VibeTorrent Kurulumu"</h3>
                    <p class="text-sm text-muted-foreground">"Yönetici hesabınızı oluşturun"</p>
                </div>

                <div class="p-6 pt-4">
                    <form on:submit=handle_setup class="space-y-4">
                        <div class="space-y-2">
                            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                "Yönetici Kullanıcı Adı"
                            </label>
                            <input 
                                type="text" 
                                placeholder="admin" 
                                class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50" 
                                prop:value=move || username.0.get()
                                on:input=move |ev| username.1.set(event_target_value(&ev))
                                disabled=move || loading.0.get()
                                required 
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                "Şifre"
                            </label>
                            <input 
                                type="password" 
                                placeholder="******" 
                                class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50" 
                                prop:value=move || password.0.get()
                                on:input=move |ev| password.1.set(event_target_value(&ev))
                                disabled=move || loading.0.get()
                                required 
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                "Şifre Onay"
                            </label>
                            <input 
                                type="password" 
                                placeholder="******" 
                                class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50" 
                                prop:value=move || confirm_password.0.get()
                                on:input=move |ev| confirm_password.1.set(event_target_value(&ev))
                                disabled=move || loading.0.get()
                                required 
                            />
                        </div>

                        <Show when=move || error.0.get().is_some() fallback=|| ()>
                            <div class="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive dark:border-destructive dark:bg-destructive/50 dark:text-destructive-foreground">
                                <span>{move || error.0.get().unwrap_or_default()}</span>
                            </div>
                        </Show>

                        <div class="pt-2">
                            <button 
                                class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground shadow hover:bg-primary/90 h-9 px-4 py-2 w-full" 
                                type="submit"
                                disabled=move || loading.0.get()
                            >
                                <Show when=move || loading.0.get() fallback=|| "Kurulumu Tamamla">
                                    <span class="animate-spin mr-2 h-4 w-4 border-2 border-current border-t-transparent rounded-full"></span>
                                    "Kuruluyor..."
                                </Show>
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}
