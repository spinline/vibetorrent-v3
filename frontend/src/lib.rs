mod app;
// mod models; // Removed
mod components;
pub mod utils;

use leptos::*;
use wasm_bindgen::prelude::*;
use app::App;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    if let Some(loader) = document.get_element_by_id("app-loading") {
        loader.remove();
    }

    mount_to_body(|| view! { <App/> })
}
