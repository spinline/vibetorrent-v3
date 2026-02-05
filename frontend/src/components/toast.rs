use leptos::*;
use shared::NotificationLevel;

// ============================================================================
// Toast Components - DaisyUI Alert Style
// ============================================================================

/// Returns the DaisyUI alert class for the notification level
fn get_alert_class(level: &NotificationLevel) -> &'static str {
    match level {
        NotificationLevel::Info => "alert-info",
        NotificationLevel::Success => "alert-success",
        NotificationLevel::Warning => "alert-warning",
        NotificationLevel::Error => "alert-error",
    }
}

/// Individual toast item component
#[component]
fn ToastItem(
    level: NotificationLevel,
    message: String,
) -> impl IntoView {
    let alert_class = get_alert_class(&level);
    
    let icon = match level {
        NotificationLevel::Info => "ℹ️",
        NotificationLevel::Success => "✓",
        NotificationLevel::Warning => "⚠️",
        NotificationLevel::Error => "✕",
    };
    
    view! {
        <div 
            class={format!("alert {} shadow-xl", alert_class)}
            style="min-width: 300px; padding: 12px 16px;"
        >
            <span style="font-size: 18px;">{icon}</span>
            <span style="font-size: 14px; font-weight: 500;">{message}</span>
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
