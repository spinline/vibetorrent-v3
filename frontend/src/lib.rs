mod app;
mod models;
mod components;

use leptos::*;
use wasm_bindgen::prelude::*;
use app::App;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    mount_to_body(|| view! { <App/> })
}
