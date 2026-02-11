use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn TableWrapper(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("overflow-hidden rounded-md border w-full", class);
    view! { <div class=class>{children()}</div> }
}

#[component]
pub fn Table(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("w-full text-sm border-collapse", class);
    view! { <table class=class>{children()}</table> }
}

#[component]
pub fn TableCaption(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("mt-4 text-sm text-muted-foreground", class);
    view! { <caption class=class>{children()}</caption> }
}

#[component]
pub fn TableHeader(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("[&_tr]:border-b bg-muted/50", class);
    view! { <thead class=class>{children()}</thead> }
}

#[component]
pub fn TableRow(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("border-b transition-colors data-[state=selected]:bg-muted hover:bg-muted/50", class);
    view! { <tr class=class>{children()}</tr> }
}

#[component]
pub fn TableHead(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("h-10 px-4 text-left align-middle font-medium text-muted-foreground whitespace-nowrap", class);
    view! { <th class=class>{children()}</th> }
}

#[component]
pub fn TableBody(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("[&_tr:last-child]:border-0", class);
    view! { <tbody class=class>{children()}</tbody> }
}

#[component]
pub fn TableCell(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("p-2 px-4 align-middle", class);
    view! { <td class=class>{children()}</td> }
}

#[component]
pub fn TableFooter(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("border-t bg-muted/50 font-medium [&>tr]:last:border-b-0", class);
    view! { <tfoot class=class>{children()}</tfoot> }
}
