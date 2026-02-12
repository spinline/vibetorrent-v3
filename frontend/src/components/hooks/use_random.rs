use leptos::prelude::*;

pub fn use_random_id_for(prefix: &str) -> String {
    format!("{}_{}", prefix, js_sys::Math::random().to_string().replace(".", ""))
}
