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

#[derive(Clone, Copy, PartialEq, Eq, Default, strum::Display, Debug)]
pub enum SonnerDirection {
    TopDown,
    #[default]
    BottomUp,
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
    #[prop(into, optional)] class: String,
    #[prop(optional, default = ToastType::default())] variant: ToastType,
    #[prop(into)] title: String,
    description: Option<String>,
    #[prop(into, optional)] position: String,
    on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    let variant_classes = match variant {
        ToastType::Default => "bg-primary text-primary-foreground shadow-xs hover:bg-primary/90",
        ToastType::Success => "bg-green-500 text-white hover:bg-green-600",
        ToastType::Error => "bg-red-500 text-white shadow-xs hover:bg-red-600",
        ToastType::Warning => "bg-yellow-500 text-white hover:bg-yellow-600",
        ToastType::Info => "bg-blue-500 text-white shadow-xs hover:bg-blue-600",
        ToastType::Loading => "bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80",
    };

    let merged_class = tw_merge!(
        "inline-flex flex-col items-start justify-center gap-1 min-w-[300px] rounded-md text-sm font-medium transition-all shadow-lg p-4 cursor-pointer pointer-events-auto border border-border/50",
        variant_classes,
        class
    );

    // Only set position attribute if not empty
    let position_attr = if position.is_empty() { None } else { Some(position) };

    // Clone title for data attribute usage, original moved into view
    let title_clone = title.clone();

    view! {
        <div
            class=merged_class
            data-name="SonnerTrigger"
            data-variant=variant.to_string()
            data-toast-title=title_clone
            data-toast-position=position_attr
            on:click=move |_| {
                if let Some(cb) = on_dismiss {
                    cb.run(());
                }
            }
        >
            <div class="font-semibold">{title}</div>
            {move || description.as_ref().map(|d| view! { <div class="text-xs opacity-90">{d.clone()}</div> })}
        </div>
    }
}

#[component]
pub fn SonnerContainer(
    children: Children,
    #[prop(into, optional)] class: String,
    #[prop(optional, default = SonnerPosition::default())] position: SonnerPosition,
) -> impl IntoView {
    let merged_class = tw_merge!("toast__container fixed z-[9999] flex flex-col gap-2 p-4 outline-none pointer-events-none", class);

    view! {
        <div class=merged_class data-position=position.to_string()>
            {children()}
        </div>
    }
}

#[component]
pub fn SonnerList(
    children: Children,
    #[prop(into, optional)] class: String,
    #[prop(optional, default = SonnerPosition::default())] position: SonnerPosition,
    #[prop(optional, default = SonnerDirection::default())] direction: SonnerDirection,
    #[prop(into, default = "false".to_string())] expanded: String,
    #[prop(into, optional)] style: String,
) -> impl IntoView {
    let merged_class = tw_merge!(
        "contents",
        class
    );

    view! {
        <div
            class=merged_class
            data-name="SonnerList"
            data-sonner-toaster="true"
            data-sonner-theme="light"
            data-position=position.to_string()
            data-expanded=expanded
            data-direction=direction.to_string()
            style=style
        >
            {children()}
        </div>
    }
}

// Thread local storage for global access without Context
thread_local! {
    static TOASTS: std::cell::RefCell<Option<RwSignal<Vec<ToastData>>>> = std::cell::RefCell::new(None);
}

pub fn provide_toaster() {
    let toasts = RwSignal::new(Vec::<ToastData>::new());
    
    // Set global thread_local
    TOASTS.with(|t| *t.borrow_mut() = Some(toasts));
    
    // Also provide context for components
    provide_context(ToasterStore { toasts });
}

#[component]
pub fn Toaster(#[prop(default = SonnerPosition::default())] position: SonnerPosition) -> impl IntoView {
    // Global store'u al
    let store = use_context::<ToasterStore>().expect("Toaster context not found. Call provide_toaster() in App root.");
    let toasts = store.toasts;

    // Auto-derive direction from position
    let direction = match position {
        SonnerPosition::TopLeft | SonnerPosition::TopCenter | SonnerPosition::TopRight => SonnerDirection::TopDown,
        _ => SonnerDirection::BottomUp,
    };

    let container_class = match position {
        SonnerPosition::TopLeft => "left-0 top-0 items-start",
        SonnerPosition::TopRight => "right-0 top-0 items-end",
        SonnerPosition::TopCenter => "left-1/2 -translate-x-1/2 top-0 items-center",
        SonnerPosition::BottomCenter => "left-1/2 -translate-x-1/2 bottom-0 items-center",
        SonnerPosition::BottomLeft => "left-0 bottom-0 items-start",
        SonnerPosition::BottomRight => "right-0 bottom-0 items-end",
    };

    view! {
        <SonnerContainer class=container_class position=position>
            <SonnerList position=position direction=direction>
                <For
                    each=move || toasts.get()
                    key=|toast| toast.id
                    children=move |toast| {
                        let id = toast.id;
                        view! {
                            <SonnerTrigger
                                variant=toast.variant
                                title=toast.title
                                description=toast.description
                                on_dismiss=Some(Callback::new(move |_| {
                                    toasts.update(|vec| vec.retain(|t| t.id != id));
                                }))
                            />
                        }
                    }
                />
            </SonnerList>
        </SonnerContainer>
    }
}

// Global Helper Functions
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
        
        toasts.update(|t| t.push(new_toast.clone()));

        // Auto remove after duration
        let duration = new_toast.duration;
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(duration as u32).await;
            toasts.update(|vec| vec.retain(|t| t.id != id));
        });
    } else {
        gloo_console::warn!("ToasterStore not found (global static). Make sure provide_toaster() is called.");
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