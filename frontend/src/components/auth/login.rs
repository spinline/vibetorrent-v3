use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::components::ui::card::{Card, CardHeader, CardContent};
use crate::components::ui::input::{Input, InputType};

use crate::components::ui::button::Button;

#[component]
pub fn Login() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = signal(Option::<String>::None);
    let loading = signal(false);

    let handle_login = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        loading.1.set(true);
        error.1.set(None);

        let user = username.get();
        let pass = password.get();

        spawn_local(async move {
            match shared::server_fns::auth::login(user, pass).await {
                Ok(_) => {
                    let window = web_sys::window().expect("window should exist");
                    let _ = window.location().set_href("/");
                }
                Err(_) => {
                    error.1.set(Some("Geçersiz kullanıcı adı veya şifre".to_string()));
                    loading.1.set(false);
                }
            }
        });
    };

    view! {
        <div class="flex items-center justify-center min-h-screen bg-muted/40 px-4">
            <Card class="w-full max-w-sm shadow-lg">
                <CardHeader class="pb-2 items-center">
                    <div class="w-12 h-12 bg-primary rounded-xl flex items-center justify-center text-primary-foreground shadow-sm mb-4">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.362 5.214A8.252 8.252 0 0112 21 8.25 8.25 0 016.038 7.048 8.287 8.287 0 009 9.6a8.983 8.983 0 013.361-6.867 8.21 8.25 0 003 2.48z" />
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 18a3.75 3.75 0 00.495-7.467 5.99 5.99 0 00-1.925 3.546 5.974 5.974 0 01-2.133-1A3.75 3.75 0 0012 18z" />
                        </svg>
                    </div>
                    <h3 class="font-semibold tracking-tight text-2xl">"VibeTorrent"</h3>
                    <p class="text-sm text-muted-foreground">"Hesabınıza giriş yapın"</p>
                </CardHeader>
                
                <CardContent class="pt-4">
                    <form on:submit=handle_login class="space-y-4">
                        <div class="space-y-2">
                            <label class="text-sm font-medium leading-none">"Kullanıcı Adı"</label>
                            <Input
                                r#type=InputType::Text
                                placeholder="Kullanıcı adınız"
                                bind_value=username
                                disabled=loading.0.get()
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium leading-none">"Şifre"</label>
                            <Input
                                r#type=InputType::Password
                                placeholder="******"
                                bind_value=password
                                disabled=loading.0.get()
                            />
                        </div>

                        <Show when=move || error.0.get().is_some()>
                            <div class="rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
                                {move || error.0.get().unwrap_or_default()}
                            </div>
                        </Show>

                        <div class="pt-2">
                            <Button
                                class="w-full"
                                attr:r#type="submit"
                                attr:disabled=move || loading.0.get()
                            >
                                <Show when=move || loading.0.get() fallback=|| view! { "Giriş Yap" }.into_any()>
                                    <span class="animate-spin mr-2 h-4 w-4 border-2 border-current border-t-transparent rounded-full"></span>
                                    "Giriş Yapılıyor..."
                                </Show>
                            </Button>
                        </div>
                    </form>
                </CardContent>
            </Card>
        </div>
    }
}