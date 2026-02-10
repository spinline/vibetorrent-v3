use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_shadcn_card::{Card, CardHeader, CardContent};
use leptos_shadcn_input::Input;
use leptos_shadcn_button::Button;
use leptos_shadcn_label::Label;
use leptos_shadcn_alert::{Alert, AlertDescription, AlertVariant};

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
            <Card class="w-full max-w-md shadow-lg overflow-hidden">
                <CardHeader class="pb-2 items-center text-center">
                    <div class="w-12 h-12 bg-primary rounded-xl flex items-center justify-center text-primary-foreground shadow-sm mb-4">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M11.42 15.17L17.25 21A2.652 2.652 0 0021 17.25l-5.877-5.877M11.42 15.17l2.496-3.03c.317-.384.74-.626 1.208-.766M11.42 15.17l-4.655 5.653a2.548 2.548 0 11-3.586-3.586l6.837-5.63m5.108-3.497a2.548 2.548 0 113.586 3.586l-6.837 5.63m-5.108 3.497l2.496-3.03c.317-.384.74-.626 1.208-.766M15.75 9.25a2.548 2.548 0 11-5.096 0 2.548 2.548 0 015.096 0z" />
                        </svg>
                    </div>
                    <h3 class="font-semibold tracking-tight text-2xl">"VibeTorrent Kurulumu"</h3>
                    <p class="text-sm text-muted-foreground">"Yönetici hesabınızı oluşturun"</p>
                </CardHeader>

                <CardContent class="pt-4">
                    <form on:submit=handle_setup class="space-y-4">
                        <div class="space-y-2">
                            <Label>"Yönetici Kullanıcı Adı"</Label>
                            <Input
                                input_type="text"
                                placeholder="admin"
                                value=MaybeProp::derive(move || Some(username.0.get()))
                                on_change=Callback::new(move |val: String| username.1.set(val))
                                disabled=Signal::derive(move || loading.0.get())
                            />
                        </div>
                        <div class="space-y-2">
                            <Label>"Şifre"</Label>
                            <Input
                                input_type="password"
                                placeholder="******"
                                value=MaybeProp::derive(move || Some(password.0.get()))
                                on_change=Callback::new(move |val: String| password.1.set(val))
                                disabled=Signal::derive(move || loading.0.get())
                            />
                        </div>
                        <div class="space-y-2">
                            <Label>"Şifre Onay"</Label>
                            <Input
                                input_type="password"
                                placeholder="******"
                                value=MaybeProp::derive(move || Some(confirm_password.0.get()))
                                on_change=Callback::new(move |val: String| confirm_password.1.set(val))
                                disabled=Signal::derive(move || loading.0.get())
                            />
                        </div>

                        <Show when=move || error.0.get().is_some() fallback=|| ()>
                            <Alert variant=AlertVariant::Destructive>
                                <AlertDescription>
                                    <span>{move || error.0.get().unwrap_or_default()}</span>
                                </AlertDescription>
                            </Alert>
                        </Show>

                        <div class="pt-2">
                            <Button
                                class="w-full"
                                disabled=Signal::derive(move || loading.0.get())
                            >
                                <Show when=move || loading.0.get() fallback=|| "Kurulumu Tamamla">
                                    <span class="animate-spin mr-2 h-4 w-4 border-2 border-current border-t-transparent rounded-full"></span>
                                    "Kuruluyor..."
                                </Show>
                            </Button>
                        </div>
                    </form>
                </CardContent>
            </Card>
        </div>
    }
}
