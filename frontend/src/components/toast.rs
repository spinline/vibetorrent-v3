use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;
use shared::NotificationLevel;

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: String,
    pub message: String,
    pub level: NotificationLevel,
    pub visible: RwSignal<bool>,
}

#[derive(Clone, Copy)]
pub struct ToastContext {
    pub toasts: RwSignal<HashMap<String, Toast>>,
}

impl ToastContext {
    pub fn add(&self, message: impl Into<String>, level: NotificationLevel) {
        let id = Uuid::new_v4().to_string();
        let message = message.into();
        let toast = Toast {
            id: id.clone(),
            message,
            level,
            visible: RwSignal::new(true),
        };

        self.toasts.update(|m| {
            m.insert(id.clone(), toast);
        });

        // Auto remove after 5 seconds
        let toasts = self.toasts;
        let id_clone = id.clone();
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(5000).await;
            toasts.update(|m| {
                if let Some(t) = m.get(&id_clone) {
                    t.visible.set(false);
                }
            });
            // Wait for animation
            gloo_timers::future::TimeoutFuture::new(300).await;
            toasts.update(|m| {
                m.remove(&id_clone);
            });
        });
    }
}

pub fn provide_toast_context() {
    let toasts = RwSignal::new(HashMap::new());
    provide_context(ToastContext { toasts });
}

#[component]
pub fn Toaster() -> impl IntoView {
    let context = expect_context::<ToastContext>();
    
    view! {
        <div class="fixed top-4 right-4 z-[100] flex flex-col gap-2 w-full max-w-sm pointer-events-none">
            {move || {
                context.toasts.get().into_values().map(|toast| {
                    view! { <ToastItem toast=toast /> }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}

#[component]
fn ToastItem(toast: Toast) -> impl IntoView {
    let (visible, set_visible) = (toast.visible, toast.visible.write_only());
    
    let base_classes = "pointer-events-auto relative w-full rounded-lg border p-4 shadow-lg transition-all duration-300 ease-in-out";
    let color_classes = match toast.level {
        NotificationLevel::Success => "bg-green-50 text-green-900 border-green-200 dark:bg-green-900 dark:text-green-100 dark:border-green-800",
        NotificationLevel::Error => "bg-red-50 text-red-900 border-red-200 dark:bg-red-900 dark:text-red-100 dark:border-red-800",
        NotificationLevel::Warning => "bg-yellow-50 text-yellow-900 border-yellow-200 dark:bg-yellow-900 dark:text-yellow-100 dark:border-yellow-800",
        NotificationLevel::Info => "bg-blue-50 text-blue-900 border-blue-200 dark:bg-blue-900 dark:text-blue-100 dark:border-blue-800",
    };

    view! {
        <div
            class=move || format!("{} {} {}", 
                base_classes, 
                color_classes,
                if visible.get() { "opacity-100 translate-x-0" } else { "opacity-0 translate-x-full" }
            )
            role="alert"
        >
            <div class="flex items-start gap-4">
                <div class="flex-1">
                    <p class="text-sm font-medium">{&toast.message}</p>
                </div>
                <button
                    class="inline-flex shrink-0 opacity-50 hover:opacity-100 focus:opacity-100 focus:outline-none"
                    on:click=move |_| set_visible.set(false)
                >
                    <span class="sr-only">"Kapat"</span>
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-4 w-4">
                        <line x1="18" x2="6" y1="6" y2="18"></line>
                        <line x1="6" x2="18" y1="6" y2="18"></line>
                    </svg>
                </button>
            </div>
        </div>
    }
}
