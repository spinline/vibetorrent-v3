use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn SvgIcon(
    children: Children,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let class = tw_merge!("size-4", class);
    
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            {children()}
        </svg>
    }
}
