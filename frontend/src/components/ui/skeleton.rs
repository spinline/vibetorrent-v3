use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Skeleton(
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let class = tw_merge!(
        "animate-pulse rounded-md bg-muted",
        class
    );
    view! { <div class=class /> }
}
