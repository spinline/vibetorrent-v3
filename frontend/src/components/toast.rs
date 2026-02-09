use leptos::prelude::*;
use leptos::logging;
use leptos::html;
use leptos::task::spawn_local;
use shared::NotificationLevel;

// ============================================================================
// Toast Components - DaisyUI Alert Style
// ============================================================================

/// Returns the DaisyUI alert class for the notification level
fn get_alert_class(level: &NotificationLevel) -> &'static str {
    match level {
        NotificationLevel::Info => "alert alert-info",
        NotificationLevel::Success => "alert alert-success",
        NotificationLevel::Warning => "alert alert-warning",
        NotificationLevel::Error => "alert alert-error",
    }
}

/// Individual toast item component
#[component]
fn ToastItem(
    level: NotificationLevel,
    message: String,
) -> impl IntoView {
    let alert_class = get_alert_class(&level);
    
    // DaisyUI SVG icons
    let icon_svg = match level {
        NotificationLevel::Info => view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
        }.into_any(),
        NotificationLevel::Success => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
        }.into_any(),
        NotificationLevel::Warning => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path>
            </svg>
        }.into_any(),
        NotificationLevel::Error => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
        }.into_any(),
    };
    
    view! {
        <div class=alert_class>
            {icon_svg}
            <span>{message}</span>
        </div>
    }
}

/// Main toast container - renders all active notifications
#[component]
pub fn ToastContainer() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("TorrentStore not provided");
    let notifications = store.notifications;

    view! {
        <div 
            class="toast toast-end toast-bottom"
            style="position: fixed; bottom: 20px; right: 20px; z-index: 99999;"
        >
            <For
                each=move || notifications.get()
                key=|item| item.id
                children=move |item| {
                    view! {
                        <ToastItem
                            level=item.notification.level
                            message=item.notification.message
                        />
                    }
                }
            />
        </div>
    }
}
