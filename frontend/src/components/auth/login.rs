use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn Login() -> impl IntoView {
    let username = signal(String::new());
    let password = signal(String::new());
    let error = signal(Option::<String>::None);
    let loading = signal(false);

    let handle_login = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        loading.1.set(true);
        error.1.set(None);

        let user = username.0.get();
        let pass = password.0.get();

        log::info!("Attempting login for user: {}", user);

        spawn_local(async move {
            match shared::server_fns::auth::login(user, pass).await {
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
        <div class="flex items-center justify-center min-h-screen bg-muted/40">
            <div class="w-full max-w-sm rounded-xl border border-border bg-card text-card-foreground shadow-lg">
                <div class="flex flex-col space-y-1.5 p-6 pb-2 items-center">
                    <div class="w-12 h-12 bg-primary rounded-xl flex items-center justify-center text-primary-foreground shadow-sm mb-4">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.362 5.214A8.252 8.252 0 0112 21 8.25 8.25 0 016.038 7.048 8.287 8.287 0 009 9.6a8.983 8.983 0 013.361-6.867 8.21 8.25 0 003 2.48z" />
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 18a3.75 3.75 0 00.495-7.467 5.99 5.99 0 00-1.925 3.546 5.974 5.974 0 01-2.133-1A3.75 3.75 0 0012 18z" />
                        </svg>
                    </div>
                    <h3 class="font-semibold tracking-tight text-2xl">"VibeTorrent"</h3>
                    <p class="text-sm text-muted-foreground">"Hesabınıza giriş yapın"</p>
                </div>
                
                <div class="p-6 pt-4">
                    <form on:submit=handle_login class="space-y-4">
                        <div class="space-y-2">
                            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                "Kullanıcı Adı"
                            </label>
                            <input 
                                type="text" 
                                placeholder="Kullanıcı adınız" 
                                class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50" 
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
                                class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50" 
                                prop:value=move || password.0.get()
                                on:input=move |ev| password.1.set(event_target_value(&ev))
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
                                <Show when=move || loading.0.get() fallback=|| "Giriş Yap">
                                    <span class="animate-spin mr-2 h-4 w-4 border-2 border-current border-t-transparent rounded-full"></span>
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