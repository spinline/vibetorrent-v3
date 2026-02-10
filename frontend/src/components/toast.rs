use leptos::prelude::*;
use shared::NotificationLevel;
use leptos_shadcn_alert::{Alert, AlertVariant};

// ============================================================================
// Toast Components - Using ShadCN Alert
// ============================================================================

fn level_to_variant(level: &NotificationLevel) -> AlertVariant {
    match level {
        NotificationLevel::Info => AlertVariant::Default,
        NotificationLevel::Success => AlertVariant::Success,
        NotificationLevel::Warning => AlertVariant::Warning,
        NotificationLevel::Error => AlertVariant::Destructive,
    }
}

fn level_icon(level: &NotificationLevel) -> impl IntoView {
    match level {
        NotificationLevel::Info => view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5 opacity-90">
                <path stroke-linecap="round" stroke-linejoin="round" d="M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z" />
            </svg>
        }.into_any(),
        NotificationLevel::Success => view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5 opacity-90">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
        }.into_any(),
        NotificationLevel::Warning => view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5 opacity-90">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
            </svg>
        }.into_any(),
        NotificationLevel::Error => view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5 opacity-90">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
            </svg>
        }.into_any(),
    }
}

/// Individual toast item component
#[component]
fn ToastItem(
    level: NotificationLevel,
    message: String,
) -> impl IntoView {
    let variant = level_to_variant(&level);
    let icon = level_icon(&level);
    
    view! {
        <Alert variant=variant class="pointer-events-auto shadow-lg">
            <div class="flex items-center gap-3">
                {icon}
                <div class="text-sm font-medium">{message}</div>
            </div>
        </Alert>
    }
}

/// Main toast container - renders all active notifications
#[component]
pub fn ToastContainer() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("TorrentStore not provided");
    let notifications = store.notifications;

    view! {
        <div 
            class="fixed bottom-0 right-0 z-[100] flex max-h-screen w-full flex-col-reverse p-4 sm:bottom-0 sm:right-0 sm:top-auto sm:flex-col md:max-w-[420px] gap-2"
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
