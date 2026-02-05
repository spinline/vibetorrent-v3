use leptos::*;
use shared::NotificationLevel;

#[component]
pub fn ToastContainer() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");
    let notifications = store.notifications;

    view! {
        <div class="toast toast-end toast-bottom z-[9999]">
            {move || notifications.get().into_iter().map(|item| {
                let alert_class = match item.notification.level {
                    NotificationLevel::Info => "alert-info",
                    NotificationLevel::Success => "alert-success",
                    NotificationLevel::Warning => "alert-warning",
                    NotificationLevel::Error => "alert-error",
                };

                let icon = match item.notification.level {
                    NotificationLevel::Info => view! { <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg> },
                    NotificationLevel::Success => view! { <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg> },
                    NotificationLevel::Warning => view! { <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" /></svg> },
                    NotificationLevel::Error => view! { <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg> },
                };

                view! {
                    <div class={format!("alert {} shadow-lg transition-all duration-300 animate-in slide-in-from-bottom-5 fade-in", alert_class)}>
                        {icon}
                        <span>{item.notification.message}</span>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
