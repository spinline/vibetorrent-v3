use leptos::prelude::*;
use leptos_ui::void;
use tw_merge::*;

// Removed unused fake components

/* ========================================================== */
/* ✨ COMPONENTS ✨ */
/* ========================================================== */

#[component]
pub fn ScrollArea(children: Children, #[prop(into, optional)] class: String) -> impl IntoView {
    let merged_class = tw_merge!("relative overflow-hidden", class);
    view! {
        <div data-name="ScrollArea" class=merged_class>
            <ScrollAreaViewport class="pr-3 pb-3 [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar]:h-1.5 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-border/60 [&::-webkit-scrollbar-thumb]:rounded-full hover:[&::-webkit-scrollbar-thumb]:bg-border/80">{children()}</ScrollAreaViewport>
        </div>
    }
}

#[component]
pub fn ScrollAreaViewport(children: Children, #[prop(into, optional)] class: String) -> impl IntoView {
    let merged_class = tw_merge!(
        "focus-visible:ring-ring/50 size-full rounded-[inherit] transition-[color,box-shadow] outline-none focus-visible:ring-[3px] focus-visible:outline-1 overflow-auto",
        class
    );
    view! {
        <div data-name="ScrollAreaViewport" class=merged_class>
            {children()}
        </div>
    }
}

/* ========================================================== */
/* 🧬 ENUMS 🧬 */
/* ========================================================== */

// Real scrollbars are now utilized in the viewport directly.

/* ========================================================== */
/* 🧬 STRUCT 🧬 */
/* ========================================================== */

#[component]
pub fn SnapScrollArea(
    #[prop(into, default = SnapAreaVariant::default())] variant: SnapAreaVariant,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let snap_item = SnapAreaClass { variant };
    let merged_class = snap_item.with_class(class);
    view! {
        <div data-name="SnapScrollArea" class=merged_class>
            {children()}
        </div>
    }
}

#[derive(TwClass, Default)]
#[tw(class = "")]
pub struct SnapAreaClass {
    variant: SnapAreaVariant,
}

#[derive(TwVariant)]
pub enum SnapAreaVariant {
    // * snap-x by default
    #[tw(default, class = "overflow-x-auto snap-x")]
    Center,
}

/* ========================================================== */
/* 🧬 STRUCT 🧬 */
/* ========================================================== */

#[component]
pub fn SnapItem(
    #[prop(into, default = SnapVariant::default())] variant: SnapVariant,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let snap_item = SnapItemClass { variant };
    let merged_class = snap_item.with_class(class);
    view! {
        <div data-name="SnapItem" class=merged_class>
            {children()}
        </div>
    }
}

#[derive(TwClass, Default)]
#[tw(class = "shrink-0")]
pub struct SnapItemClass {
    variant: SnapVariant,
}

#[derive(TwVariant)]
pub enum SnapVariant {
    // * snap-center by default
    #[tw(default, class = "snap-center")]
    Center,
}
