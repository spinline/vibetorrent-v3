use axum::{
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    BoxError,
};
use rust_embed::RustEmbed;

pub mod auth;
pub mod setup;

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
pub struct Asset;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return StatusCode::NOT_FOUND.into_response();
            }
            match Asset::get("index.html") {
                Some(content) => {
                    let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                    ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
                }
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }
}

pub async fn handle_timeout_error(err: BoxError) -> (StatusCode, &'static str) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "Request timed out")
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unhandled internal error",
        )
    }
}

#[cfg(feature = "push-notifications")]
pub async fn get_push_public_key_handler(
    axum::extract::State(state): axum::extract::State<crate::AppState>,
) -> impl IntoResponse {
    let public_key = state.push_store.get_public_key();
    (StatusCode::OK, axum::extract::Json(serde_json::json!({ "publicKey": public_key }))).into_response()
}

#[cfg(feature = "push-notifications")]
pub async fn subscribe_push_handler(
    axum::extract::State(state): axum::extract::State<crate::AppState>,
    axum::extract::Json(subscription): axum::extract::Json<crate::push::PushSubscription>,
) -> impl IntoResponse {
    tracing::info!("Received push subscription: {:?}", subscription);
    state.push_store.add_subscription(subscription).await;
    (StatusCode::OK, "Subscription saved").into_response()
}
