mod app;
// mod models; // Removed
mod components;
pub mod utils;
pub mod store;

use leptos::*;
use wasm_bindgen::prelude::*;
use app::App;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    
    // Add app-loaded class to body to hide spinner via CSS
    let _ = body.class_list().add_1("app-loaded");

    // Also try to remove it directly
    if let Some(loader) = document.get_element_by_id("app-loading") {
        loader.remove();
    }

    mount_to_body(|| view! { <App/> })
}
