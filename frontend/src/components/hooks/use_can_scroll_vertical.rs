use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Hook to determine if an element can scroll vertically.
///
/// Returns (on_scroll_callback, can_scroll_up_signal, can_scroll_down_signal)
pub fn use_can_scroll_vertical() -> (Callback<web_sys::Event>, ReadSignal<bool>, ReadSignal<bool>) {
    let can_scroll_up = RwSignal::new(false);
    let can_scroll_down = RwSignal::new(false);

    let on_scroll = Callback::new(move |ev: web_sys::Event| {
        if let Some(target) = ev.target() {
            if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                let scroll_top = el.scroll_top();
                let scroll_height = el.scroll_height();
                let client_height = el.client_height();

                can_scroll_up.set(scroll_top > 0);
                can_scroll_down.set(scroll_top + client_height < scroll_height - 1);
            }
        }
    });

    (on_scroll, can_scroll_up.read_only(), can_scroll_down.read_only())
}
