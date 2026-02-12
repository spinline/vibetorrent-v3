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
    pub is_exiting: RwSignal<bool>,
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
    is_expanded: Signal<bool>,
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

    // Simplified Style (No manual translateY needed with Flexbox)
    let style = move || {
        format!(
            "z-index: {}; opacity: 1; transition: all 0.3s ease;",
            total - index
        )
    };

    let animation_class = move || {
        let pos = position.to_string();
        let is_left = pos.contains("Left");
        let is_exiting = toast.is_exiting.get();
        
        match (is_left, is_exiting) {
            (true, false) => "animate-sonner-in-left",
            (true, true) => "animate-sonner-out-left",
            (false, false) => "animate-sonner-in-right",
            (false, true) => "animate-sonner-out-right",
        }
    };

    let icon = match toast.variant {
        ToastType::Success => Some(view! { <span class="icon font-bold text-lg">"✓"</span> }.into_any()),
        ToastType::Error => Some(view! { <span class="icon font-bold text-lg">"✕"</span> }.into_any()),
        ToastType::Warning => Some(view! { <span class="icon font-bold text-lg">"⚠"</span> }.into_any()),
        ToastType::Info => Some(view! { <span class="icon font-bold text-lg">"ℹ"</span> }.into_any()),
        _ => None,
    };

    view! {
        <div
            class=move || tw_merge!(
                "relative transition-all duration-300 ease-in-out cursor-pointer pointer-events-auto",
                "flex items-center gap-3 w-full max-w-[calc(100vw-2rem)] sm:max-w-[380px] p-4 rounded-lg border shadow-xl bg-card",
                variant_classes,
                animation_class()
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
                <div class="text-sm font-bold truncate leading-tight">{toast.title}</div>
                {move || toast.description.as_ref().map(|d| view! { <div class="text-xs opacity-80 truncate">{d.clone()}</div> })}
            </div>

            // Progress Bar
            <div 
                class=tw_merge!("absolute bottom-0 left-0 h-1 w-full opacity-30", bar_color)
                style=format!(
                    "animation: sonner-progress {}ms linear forwards; transform-origin: left;",
                    toast.duration
                )
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

    let is_bottom = position.to_string().contains("Bottom");

    let container_class = match position {
        SonnerPosition::TopLeft => "left-6 top-6 items-start",
        SonnerPosition::TopRight => "right-6 top-6 items-end",
        SonnerPosition::TopCenter => "left-1/2 -translate-x-1/2 top-6 items-center",
        SonnerPosition::BottomCenter => "left-1/2 -translate-x-1/2 bottom-6 items-center",
        SonnerPosition::BottomLeft => "left-6 bottom-6 items-start",
        SonnerPosition::BottomRight => "right-6 bottom-6 items-end",
    };

    view! {
        <style>
            "@keyframes sonner-progress { from { transform: scaleX(1); } to { transform: scaleX(0); } }
             @keyframes sonner-in-right { from { transform: translateX(100%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
             @keyframes sonner-out-right { from { transform: translateX(0); opacity: 1; } to { transform: translateX(100%); opacity: 0; } }
             @keyframes sonner-in-left { from { transform: translateX(-100%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
             @keyframes sonner-out-left { from { transform: translateX(0); opacity: 1; } to { transform: translateX(-100%); opacity: 0; } }
             .animate-sonner-in-right { animation: sonner-in-right 0.3s ease-out forwards; }
             .animate-sonner-out-right { animation: sonner-out-right 0.3s ease-in forwards; }
             .animate-sonner-in-left { animation: sonner-in-left 0.3s ease-out forwards; }
             .animate-sonner-out-left { animation: sonner-out-left 0.3s ease-in forwards; }"
        </style>
        <div 
            class=tw_merge!(
                "fixed z-[100] flex gap-3 pointer-events-none w-full sm:w-[400px]",
                if is_bottom { "flex-col-reverse" } else { "flex-col" },
                container_class,
                "pb-[env(safe-area-inset-bottom)] pt-[env(safe-area-inset-top)] px-4 sm:px-0"
            )
        >
            <For
                each=move || {
                    let list = toasts.get();
                    list.into_iter().enumerate().collect::<Vec<_>>()
                }
                key=|(_, toast)| toast.id
                children=move |(index, toast)| {
                    let id = toast.id;
                    let total = toasts.with(|t| t.len());
                    let is_exiting = toast.is_exiting;
                    
                    view! {
                        <SonnerTrigger
                            toast=toast
                            index=index
                            total=total
                            position=position
                            is_expanded=Signal::derive(move || true)
                            on_dismiss=Callback::new(move |_| {
                                is_exiting.set(true);
                                leptos::task::spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(300).await;
                                    toasts.update(|vec| vec.retain(|t| t.id != id));
                                });
                            })
                        />
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
            is_exiting: RwSignal::new(false),
        };
        
        toasts.update(|t| {
            t.push(new_toast.clone());
            if t.len() > 5 {
                t.remove(0);
            }
        });

        let duration = new_toast.duration;
        let is_exiting = new_toast.is_exiting;
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(duration as u32).await;
            is_exiting.set(true);
            gloo_timers::future::TimeoutFuture::new(300).await;
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