use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Switch(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into, optional)] on_checked_change: Option<Callback<bool>>,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    let checked_val = move || checked.get();
    let disabled_val = move || disabled.get();

    let track_class = move || tw_merge!(
        "inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
        if checked_val() { "bg-primary" } else { "bg-input" },
        class.clone()
    );

    let thumb_class = move || tw_merge!(
        "pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform",
        if checked_val() { "translate-x-4" } else { "translate-x-0" }
    );

    view! {
        <button
            type="button"
            role="switch"
            aria-checked=move || checked_val().to_string()
            disabled=disabled_val
            class=track_class
            on:click=move |e| {
                e.prevent_default();
                if let Some(cb) = on_checked_change {
                    cb.run(!checked_val());
                }
            }
        >
            <span class=thumb_class />
        </button>
    }
}
