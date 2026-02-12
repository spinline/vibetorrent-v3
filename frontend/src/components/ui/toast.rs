use leptos::prelude::*;
use tw_merge::*;

#[derive(Clone, Copy, PartialEq, Eq, Default, strum::Display, Debug)]
#[allow(dead_code)]
pub enum ToastType {
    #[default]
    Default,
    Success,
    Error,
    Warning,
    Info,
    Loading,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, strum::Display, Debug)]
#[allow(dead_code)]
pub enum SonnerPosition {
    TopLeft,
    TopCenter,
    TopRight,
    #[default]
    BottomRight,
    BottomCenter,
    BottomLeft,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastData {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub variant: ToastType,
    pub duration: u64, // ms
}

#[derive(Clone, Copy)]
pub struct ToasterStore {
    pub toasts: RwSignal<Vec<ToastData>>,
}

#[component]
pub fn SonnerTrigger(
    toast: ToastData,
    index: usize,
    total: usize,
    position: SonnerPosition,
    #[prop(optional)] on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    let variant_classes = match toast.variant {
        ToastType::Default => "bg-background text-foreground border-border",
        ToastType::Success => "bg-background text-foreground border-border [&_.icon]:text-green-500",
        ToastType::Error => "bg-background text-foreground border-border [&_.icon]:text-destructive",
        ToastType::Warning => "bg-background text-foreground border-border [&_.icon]:text-yellow-500",
        ToastType::Info => "bg-background text-foreground border-border [&_.icon]:text-blue-500",
        ToastType::Loading => "bg-background text-foreground border-border",
    };

    let bar_color = match toast.variant {
        ToastType::Success => "bg-green-500",
        ToastType::Error => "bg-destructive",
        ToastType::Warning => "bg-yellow-500",
        ToastType::Info => "bg-blue-500",
        _ => "bg-primary",
    };

    // Sonner Stacking Logic
    let inverse_index = index; 
    let offset = inverse_index as f64 * 12.0;
    let scale = 1.0 - (inverse_index as f64 * 0.05);
    let opacity = if inverse_index > 2 { 0.0 } else { 1.0 - (inverse_index as f64 * 0.15) };
    
    let is_bottom = position.to_string().contains("Bottom");
    let y_direction = if is_bottom { -1.0 } else { 1.0 };
    let translate_y = offset * y_direction;

    let style = format!(
        "z-index: {}; transform: translateY({}px) scale({}); opacity: {};",
        total - index,
        translate_y,
        scale,
        opacity
    );

    let icon = match toast.variant {
        ToastType::Success => Some(view! { <span class="icon font-bold">"✓"</span> }.into_any()),
        ToastType::Error => Some(view! { <span class="icon font-bold">"✕"</span> }.into_any()),
        ToastType::Warning => Some(view! { <span class="icon font-bold">"⚠"</span> }.into_any()),
        ToastType::Info => Some(view! { <span class="icon font-bold">"ℹ"</span> }.into_any()),
        _ => None,
    };

    view! {
        <style>
            "
            @keyframes sonner-progress {
                from { transform: scaleX(1); }
                to { transform: scaleX(0); }
            }
            .sonner-progress-bar {
                animation: sonner-progress linear forwards;
                transform-origin: left;
            }
            "
        </style>
        <div
            class=tw_merge!(
                "absolute transition-all duration-300 ease-in-out cursor-pointer pointer-events-auto overflow-hidden",
                "flex items-center gap-3 w-full max-w-[calc(100vw-2rem)] sm:max-w-[380px] p-4 rounded-lg border shadow-lg bg-card",
                if is_bottom { "bottom-0" } else { "top-0" },
                variant_classes
            )
            style=style
            on:click=move |_| {
                if let Some(cb) = on_dismiss {
                    cb.run(());
                }
            }
        >
            {icon}
            <div class="flex flex-col gap-0.5 overflow-hidden flex-1">
                <div class="text-sm font-semibold truncate leading-tight">{toast.title}</div>
                {move || toast.description.as_ref().map(|d| view! { <div class="text-xs opacity-70 truncate">{d.clone()}</div> })}
            </div>

            // Progress Bar
            <div 
                class=tw_merge!("absolute bottom-0 left-0 h-1 w-full sonner-progress-bar opacity-40", bar_color)
                style=format!("animation-duration: {}ms;", toast.duration)
            />
        </div>
    }.into_any()
}

thread_local! {
    static TOASTS: std::cell::RefCell<Option<RwSignal<Vec<ToastData>>>> = std::cell::RefCell::new(None);
}

pub fn provide_toaster() {
    let toasts = RwSignal::new(Vec::<ToastData>::new());
    TOASTS.with(|t| *t.borrow_mut() = Some(toasts));
    provide_context(ToasterStore { toasts });
}

#[component]
pub fn Toaster(#[prop(default = SonnerPosition::default())] position: SonnerPosition) -> impl IntoView {
    let store = use_context::<ToasterStore>().expect("Toaster context not found");
    let toasts = store.toasts;
    let is_hovered = RwSignal::new(false);

    let container_class = match position {
        SonnerPosition::TopLeft => "left-6 top-6 items-start",
        SonnerPosition::TopRight => ("right-6 top-6 items-end"),
        SonnerPosition::TopCenter => ("left-1/2 -translate-x-1/2 top-6 items-center"),
        SonnerPosition::BottomCenter => ("left-1/2 -translate-x-1/2 bottom-6 items-center"),
        SonnerPosition::BottomLeft => ("left-6 bottom-6 items-start"),
        SonnerPosition::BottomRight => ("right-6 bottom-6 items-end"),
    };

    view! {
        <div 
            class=tw_merge!(
                "fixed z-[100] flex flex-col pointer-events-none min-h-[100px] w-full sm:w-[400px]",
                container_class,
                "pb-[env(safe-area-inset-bottom)] pt-[env(safe-area-inset-top)] px-4 sm:px-0"
            )
            on:mouseenter=move |_| is_hovered.set(true)
            on:mouseleave=move |_| is_hovered.set(false)
        >
            <For
                each=move || {
                    let list = toasts.get();
                    list.into_iter().rev().enumerate().collect::<Vec<_>>()
                }
                key=|(_, toast)| toast.id
                children=move |(index, toast)| {
                    let id = toast.id;
                    let total = toasts.with(|t| t.len());
                    
                    let expanded_style = move || {
                        if is_hovered.get() {
                            let offset = index as f64 * 64.0;
                            let is_bottom = position.to_string().contains("Bottom");
                            let y_dir = if is_bottom { -1.0 } else { 1.0 };
                            format!("transform: translateY({}px) scale(1); opacity: 1;", offset * y_dir)
                        } else {
                            "".to_string()
                        }
                    };

                    view! {
                        <div class="contents" style=expanded_style>
                            <SonnerTrigger
                                toast=toast
                                index=index
                                total=total
                                position=position
                                on_dismiss=Callback::new(move |_| {
                                    toasts.update(|vec| vec.retain(|t| t.id != id));
                                })
                            />
                        </div>
                    }
                }
            />
        </div>
    }.into_any()
}

pub fn toast(title: impl Into<String>, variant: ToastType) {
    let signal_opt = TOASTS.with(|t| *t.borrow());
    
    if let Some(toasts) = signal_opt {
        let id = js_sys::Math::random().to_bits();
        let new_toast = ToastData {
            id,
            title: title.into(),
            description: None,
            variant,
            duration: 4000,
        };
        
        toasts.update(|t| {
            t.push(new_toast.clone());
            if t.len() > 5 {
                t.remove(0);
            }
        });

        let duration = new_toast.duration;
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(duration as u32).await;
            toasts.update(|vec| vec.retain(|t| t.id != id));
        });
    }
}

#[allow(dead_code)]
pub fn toast_success(title: impl Into<String>) { toast(title, ToastType::Success); }
#[allow(dead_code)]
pub fn toast_error(title: impl Into<String>) { toast(title, ToastType::Error); }
#[allow(dead_code)]
pub fn toast_warning(title: impl Into<String>) { toast(title, ToastType::Warning); }
#[allow(dead_code)]
pub fn toast_info(title: impl Into<String>) { toast(title, ToastType::Info); }
