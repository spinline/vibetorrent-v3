use crate::AppState;
use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct SetupRequest {
    username: String,
    password: String,
}

#[derive(Serialize, ToSchema)]
pub struct SetupStatusResponse {
    completed: bool,
}

#[utoipa::path(
    get,
    path = "/api/setup/status",
    responses(
        (status = 200, description = "Setup status", body = SetupStatusResponse)
    )
)]
pub async fn get_setup_status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let completed = match state.db.has_users().await {
        Ok(has) => has,
        Err(e) => {
            tracing::error!("DB error checking users: {}", e);
            false
        }
    };
    Json(SetupStatusResponse { completed }).into_response()
}

#[utoipa::path(
    post,
    path = "/api/setup",
    request_body = SetupRequest,
    responses(
        (status = 200, description = "Setup completed"),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Setup already completed"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn setup_handler(
    State(state): State<AppState>,
    Json(payload): Json<SetupRequest>,
) -> impl IntoResponse {
    // 1. Check if setup is already completed (i.e., users exist)
    match state.db.has_users().await {
        Ok(true) => return (StatusCode::FORBIDDEN, "Setup already completed").into_response(),
        Err(e) => {
            tracing::error!("DB error checking users: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
        Ok(false) => {} // Proceed
    }

    // 2. Validate input
    if payload.username.len() < 3 || payload.password.len() < 6 {
        return (StatusCode::BAD_REQUEST, "Username must be at least 3 chars, password at least 6").into_response();
    }

    // 3. Create User
    let password_hash = match bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Failed to hash password: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to process password").into_response();
        }
    };

    if let Err(e) = state.db.create_user(&payload.username, &password_hash).await {
        tracing::error!("Failed to create user: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response();
    }

    (StatusCode::OK, "Setup completed successfully").into_response()
}
