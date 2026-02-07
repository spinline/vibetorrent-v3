use crate::AppState;
use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use time::Duration;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[allow(dead_code)]
#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    username: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful"),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    tracing::info!("Login attempt for user: {}", payload.username);

    let user = match state.db.get_user_by_username(&payload.username).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::warn!("Login failed: User not found for {}", payload.username);
            return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
        }
        Err(e) => {
            tracing::error!("DB error during login for {}: {}", payload.username, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let (user_id, password_hash) = user;

    match bcrypt::verify(&payload.password, &password_hash) {
        Ok(true) => {
            tracing::info!("Password verified for user: {}", payload.username);

            // Create session
            let token: String = (0..32).map(|_| {
                use rand::{distributions::Alphanumeric, Rng};
                rand::thread_rng().sample(Alphanumeric) as char
            }).collect();

            // Expires in 30 days
            let expires_in = 60 * 60 * 24 * 30;
            let expires_at = time::OffsetDateTime::now_utc().unix_timestamp() + expires_in;

            if let Err(e) = state.db.create_session(user_id, &token, expires_at).await {
                tracing::error!("Failed to create session for {}: {}", payload.username, e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session").into_response();
            }

            let cookie = Cookie::build(("auth_token", token))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(Duration::seconds(expires_in))
                .build();

            tracing::info!("Session created and cookie set for user: {}", payload.username);
            (StatusCode::OK, jar.add(cookie), "Login successful").into_response()
        }
        Ok(false) => {
            tracing::warn!("Login failed: Invalid password for {}", payload.username);
            (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
        }
        Err(e) => {
            tracing::error!("Bcrypt error for {}: {}", payload.username, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Auth error").into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logged out")
    )
)]
pub async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(token) = jar.get("auth_token") {
        let _ = state.db.delete_session(token.value()).await;
    }

    let cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .http_only(true)
        .max_age(Duration::seconds(-1)) // Expire immediately
        .build();

    (StatusCode::OK, jar.add(cookie), "Logged out").into_response()
}

#[utoipa::path(
    get,
    path = "/api/auth/check",
    responses(
        (status = 200, description = "Authenticated"),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn check_auth_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(token) = jar.get("auth_token") {
        match state.db.get_session_user(token.value()).await {
            Ok(Some(_)) => return StatusCode::OK.into_response(),
            _ => {} // Invalid session
        }
    }

    StatusCode::UNAUTHORIZED.into_response()
}
