use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Shimmer(
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let merged_class = tw_merge!(
        "relative overflow-hidden bg-muted before:absolute before:inset-0 before:-translate-x-full before:animate-[shimmer_2s_infinite] before:bg-gradient-to-r before:from-transparent before:via-white/20 before:to-transparent dark:before:via-white/5",
        class
    );
    view! { <div class=merged_class /> }
}
