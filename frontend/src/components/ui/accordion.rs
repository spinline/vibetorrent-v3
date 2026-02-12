use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Accordion(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("w-full", class);
    view! { <div class=class>{children()}</div> }
}

#[component]
pub fn AccordionItem(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("border-b", class);
    view! { <div class=class>{children()}</div> }
}

#[component]
pub fn AccordionHeader(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("flex", class);
    view! { <div class=class>{children()}</div> }
}

#[component]
pub fn AccordionTrigger(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!(
        "flex flex-1 items-center justify-between py-4 font-medium transition-all hover:underline [&[data-state=open]>svg]:rotate-180",
        class
    );
    view! {
        <button type="button" class=class>
            {children()}
        </button>
    }
}

#[component]
pub fn AccordionContent(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("overflow-hidden text-sm transition-all", class);
    view! { <div class=class>{children()}</div> }
}
