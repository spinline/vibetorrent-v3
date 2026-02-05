use leptos::*;
use shared::NotificationLevel;

// ============================================================================
// Toast Icons (Clean Code: Separation of Concerns)
// ============================================================================

/// Returns the appropriate SVG icon for the notification level
fn get_toast_icon(level: &NotificationLevel) -> impl IntoView {
    match level {
        NotificationLevel::Info => view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-5 h-5">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
        }.into_view(),
        NotificationLevel::Success => view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-5 h-5">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
        }.into_view(),
        NotificationLevel::Warning => view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-5 h-5">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
        }.into_view(),
        NotificationLevel::Error => view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-5 h-5">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
        }.into_view(),
    }
}

/// Returns the DaisyUI alert class for the notification level
fn get_alert_class(level: &NotificationLevel) -> &'static str {
    match level {
        NotificationLevel::Info => "alert-info",
        NotificationLevel::Success => "alert-success",
        NotificationLevel::Warning => "alert-warning",
        NotificationLevel::Error => "alert-error",
    }
}

// ============================================================================
// Toast Components (Clean Code: Single Responsibility)
// ============================================================================

/// Individual toast item component
#[component]
fn ToastItem(
    level: NotificationLevel,
    message: String,
) -> impl IntoView {
    let alert_class = get_alert_class(&level);
    
    view! {
        <div class={format!(
            "alert {} shadow-lg min-w-[280px] max-w-[400px] transition-all duration-300 animate-in slide-in-from-right fade-in",
            alert_class
        )}>
            {get_toast_icon(&level)}
            <span class="text-sm font-medium truncate">{message}</span>
        </div>
    }
}

/// Main toast container - renders all active notifications
#[component]
pub fn ToastContainer() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("TorrentStore not provided");
    let notifications = store.notifications;

    view! {
        // Fixed to viewport with explicit inset values for reliable positioning
        <div 
            class="fixed flex flex-col gap-2 items-end pointer-events-none"
            style="bottom: 16px; right: 16px; z-index: 99999;"
        >
            <For
                each=move || notifications.get()
                key=|item| item.id
                children=move |item| {
                    view! {
                        <div class="pointer-events-auto">
                            <ToastItem
                                level=item.notification.level
                                message=item.notification.message
                            />
                        </div>
                    }
                }
            />
        </div>
    }
}
