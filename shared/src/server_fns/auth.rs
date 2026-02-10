use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub uid: i64,    // user id
    pub exp: usize,
}

#[server(Login, "/api/auth/login")]
pub async fn login(username: String, password: String) -> Result<UserResponse, ServerFnError> {
    use crate::DbContext;
    use leptos_axum::ResponseOptions;
    use jsonwebtoken::{encode, Header, EncodingKey};
    use cookie::{Cookie, SameSite};
    use std::time::{SystemTime, UNIX_EPOCH};

    let db_context = use_context::<DbContext>().ok_or(ServerFnError::ServerError("DB Context missing".to_string()))?;
    
    let user_opt = db_context.db.get_user_by_username(&username).await
        .map_err(|e| ServerFnError::ServerError(format!("DB error: {}", e)))?;

    if let Some((uid, password_hash)) = user_opt {
        let valid = bcrypt::verify(&password, &password_hash).unwrap_or(false);
        if !valid {
             return Err(ServerFnError::ServerError("Invalid credentials".to_string()));
        }

        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 24 * 3600; // 24 hours

        let claims = Claims {
            sub: username.clone(),
            uid,
            exp: expiration,
        };

        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
            .map_err(|e| ServerFnError::ServerError(format!("Token error: {}", e)))?;

        let cookie = Cookie::build(("auth_token", token))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Strict)
            .build();

        if let Some(options) = use_context::<ResponseOptions>() {
            options.insert_header(
                axum::http::header::SET_COOKIE,
                axum::http::HeaderValue::from_str(&cookie.to_string()).unwrap(),
            );
        }

        Ok(UserResponse {
            id: uid,
            username,
        })
    } else {
        Err(ServerFnError::ServerError("Invalid credentials".to_string()))
    }
}

#[server(Logout, "/api/auth/logout")]
pub async fn logout() -> Result<(), ServerFnError> {
    use leptos_axum::ResponseOptions;
    use cookie::{Cookie, SameSite};

    let cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(cookie::time::Duration::seconds(0))
        .build();

    if let Some(options) = use_context::<ResponseOptions>() {
        options.insert_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_str(&cookie.to_string()).unwrap(),
        );
    }
    Ok(())
}

#[server(GetUser, "/api/auth/user")]
pub async fn get_user() -> Result<Option<UserResponse>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    use jsonwebtoken::{decode, Validation, DecodingKey};

    let headers: HeaderMap = extract().await?;
    let cookie_header = headers.get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok());

    if let Some(cookie_str) = cookie_header {
        // Parse all cookies
        for c_str in cookie_str.split(';') {
            if let Ok(c) = cookie::Cookie::parse(c_str.trim()) {
                if c.name() == "auth_token" {
                    let token = c.value();
                    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
                    let token_data = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(secret.as_bytes()),
                        &Validation::default(),
                    );

                    if let Ok(data) = token_data {
                        return Ok(Some(UserResponse {
                            id: data.claims.uid,
                            username: data.claims.sub,
                        }));
                    }
                }
            }
        }
    }

    Ok(None)
}
