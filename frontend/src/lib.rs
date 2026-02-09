mod app;
mod components;
pub mod utils;
pub mod store;
pub mod api;

use leptos::prelude::*;
use leptos::mount::mount_to_body;
use wasm_bindgen::prelude::*;
use app::App;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug)
        .expect("Failed to initialize logging");

    let window = web_sys::window()
        .expect("Failed to access window - browser may not be fully loaded");
    let document = window.document()
        .expect("Failed to access document");
    let body = document.body()
        .expect("Failed to access document body");
    
    // Add app-loaded class to body to hide spinner via CSS
    let _ = body.class_list().add_1("app-loaded");

    // Also try to remove it directly
    if let Some(loader) = document.get_element_by_id("app-loading") {
        loader.remove();
    }

    mount_to_body(|| view! { <App/> })
}
