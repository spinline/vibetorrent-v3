use leptos::prelude::*;
use leptos_ui::clx;

mod components {
    use super::*;

    clx! {Card, div, "bg-card text-card-foreground flex flex-col gap-4 rounded-xl border py-6 shadow-sm"}
    clx! {CardHeader, div, "@container/card-header flex flex-col items-start gap-1.5 px-6 [.border-b]:pb-6"}
    clx! {CardTitle, h2, "leading-none font-semibold"}
    clx! {CardContent, div, "px-6"}
    clx! {CardDescription, p, "text-muted-foreground text-sm"}
    clx! {CardFooter, footer, "flex items-center px-6 [.border-t]:pt-6", "gap-2"}
}

pub use components::*;
