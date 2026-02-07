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
        (status = 200, description = "Setup completed and logged in"),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Setup already completed"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn setup_handler(
    State(state): State<AppState>,
    jar: CookieJar,
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
    // Lower cost for faster login on low-power devices (MIPS routers etc.)
    let password_hash = match bcrypt::hash(&payload.password, 6) {
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

    // 4. Auto-Login (Create Session)
    // Get the created user's ID
    let user = match state.db.get_user_by_username(&payload.username).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::INTERNAL_SERVER_ERROR, "User created but not found").into_response(),
        Err(e) => {
            tracing::error!("DB error fetching new user: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };
    let (user_id, _) = user;

    // Create session token
    let token: String = (0..32).map(|_| {
        use rand::{distributions::Alphanumeric, Rng};
        rand::thread_rng().sample(Alphanumeric) as char
    }).collect();

    // Default expiration: 1 day (since it's not "remember me")
    let expires_in = 60 * 60 * 24;
    let expires_at = time::OffsetDateTime::now_utc().unix_timestamp() + expires_in;

    if let Err(e) = state.db.create_session(user_id, &token, expires_at).await {
        tracing::error!("Failed to create session for new user: {}", e);
        // Even if session fails, setup is technically complete, but login failed.
        // We return OK but user will have to login manually.
        return (StatusCode::OK, "Setup completed, please login").into_response();
    }

    let mut cookie = Cookie::build(("auth_token", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    cookie.set_max_age(Duration::seconds(expires_in));

    (StatusCode::OK, jar.add(cookie), "Setup completed and logged in").into_response()
}
