use leptos::context::Provider;
use leptos::prelude::*;
use tw_merge::tw_merge;

#[derive(Clone)]
pub struct TabsContext {
    pub active_tab: RwSignal<String>,
}

#[component]
pub fn Tabs(
    #[prop(into)] default_value: String,
    children: Children,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let active_tab = RwSignal::new(default_value);
    let ctx = TabsContext { active_tab };

    let merged_class = tw_merge!("w-full", &class);

    view! {
        <Provider value=ctx>
            <div data-name="Tabs" class=merged_class>
                {children()}
            </div>
        </Provider>
    }
}

#[component]
pub fn TabsList(
    children: Children,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let merged_class = tw_merge!(
        "inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground",
        &class
    );

    view! {
        <div data-name="TabsList" class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn TabsTrigger(
    #[prop(into)] value: String,
    children: Children,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let ctx = expect_context::<TabsContext>();
    let v_clone = value.clone();
    
    let is_active = Memo::new(move |_| ctx.active_tab.get() == v_clone);

    let merged_class = move || tw_merge!(
        "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 cursor-pointer select-none",
        if is_active.get() {
            "bg-background text-foreground shadow-sm"
        } else {
            "hover:bg-background/50 hover:text-foreground"
        },
        &class
    );

    view! {
        <button
            data-name="TabsTrigger"
            type="button"
            class=merged_class
            data-state=move || if is_active.get() { "active" } else { "inactive" }
            on:click=move |_| ctx.active_tab.set(value.clone())
        >
            {children()}
        </button>
    }
}

#[component]
pub fn TabsContent(
    #[prop(into)] value: String,
    children: Children,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let ctx = expect_context::<TabsContext>();
    let v_clone = value.clone();
    
    let is_active = Memo::new(move |_| ctx.active_tab.get() == v_clone);

    let merged_class = move || tw_merge!(
        "mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        if !is_active.get() { "hidden" } else { "" },
        &class
    );

    view! {
        <div
            data-name="TabsContent"
            class=merged_class
            data-state=move || if is_active.get() { "active" } else { "inactive" }
            tabindex=move || if is_active.get() { "0" } else { "-1" }
        >
            {children()}
        </div>
    }
}
