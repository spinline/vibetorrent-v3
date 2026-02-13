use leptos::prelude::*;
use tailwind_fuse::tw_merge;
use crate::components::ui::button::{Button, ButtonVariant};

#[component]
pub fn ButtonAction(
    children: Children,
    #[prop(into)] on_action: Callback<()>,
    #[prop(optional, into)] class: String,
    #[prop(default = 1000)] hold_duration: u64,
    #[prop(default = ButtonVariant::Default)] variant: ButtonVariant,
) -> impl IntoView {
    let is_holding = RwSignal::new(false);
    
    let on_down = move |_| is_holding.set(true);
    let on_up = move |_| is_holding.set(false);

    Effect::new(move |_| {
        if is_holding.get() {
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(hold_duration as u32).await;
                if is_holding.get_untracked() {
                    on_action.run(());
                    is_holding.set(false);
                }
            });
        }
    });

    let merged_class = move || tw_merge!(
        "relative overflow-hidden transition-all active:scale-[0.98]",
        class.clone()
    );

    view! {
        <style>
            "@keyframes button-hold-progress {
                from { width: 0%; }
                to { width: 100%; }
            }
            .animate-button-hold {
                animation: button-hold-progress var(--button-hold-duration) linear forwards;
            }"
        </style>
        <Button
            variant=variant
            class=merged_class()
            attr:style=format!("--button-hold-duration: {}ms", hold_duration)
            on:mousedown=on_down
            on:mouseup=on_up
            on:mouseleave=on_up
            on:touchstart=move |_| is_holding.set(true)
            on:touchend=on_up
        >
            // Progress Overlay
            <Show when=move || is_holding.get()>
                <div class="absolute inset-0 bg-white/20 dark:bg-black/20 pointer-events-none animate-button-hold" />
            </Show>
            
            <span class="relative z-10 flex items-center justify-center gap-2">
                {children()}
            </span>
        </Button>
    }
}
