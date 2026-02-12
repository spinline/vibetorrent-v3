use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[derive(Clone, Copy, PartialEq, Eq, Default, strum::Display)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Outline,
    Destructive,
    Success,
    Warning,
    Info,
}

#[component]
pub fn Badge(
    children: Children,
    #[prop(optional, default = BadgeVariant::Default)] variant: BadgeVariant,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let variant_classes = match variant {
        BadgeVariant::Default => "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
        BadgeVariant::Secondary => "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
        BadgeVariant::Outline => "text-foreground",
        BadgeVariant::Destructive => "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
        BadgeVariant::Success => "border-transparent bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20",
        BadgeVariant::Warning => "border-transparent bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 border-yellow-500/20",
        BadgeVariant::Info => "border-transparent bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/20",
    };

    let class = tw_merge!(
        "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
        variant_classes,
        class
    );

    view! {
        <div class=class>
            {children()}
        </div>
    }
}
