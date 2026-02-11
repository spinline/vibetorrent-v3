use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use crate::codec::MessagePack;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetupStatus {
    pub completed: bool,
}

#[server(GetSetupStatus, "/api/server_fns/GetSetupStatus", encoding = "MessagePack")]
pub async fn get_setup_status() -> Result<SetupStatus, ServerFnError> {
    use crate::DbContext;

    let db_context = use_context::<DbContext>().ok_or_else(|| ServerFnError::new("DB Context missing"))?;
    let has_users = db_context.db.has_users().await
        .map_err(|e| ServerFnError::new(format!("DB error: {}", e)))?;

    Ok(SetupStatus {
        completed: has_users,
    })
}

#[server(Setup, "/api/server_fns/Setup", encoding = "MessagePack")]
pub async fn setup(username: String, password: String) -> Result<(), ServerFnError> {
    use crate::DbContext;

    let db_context = use_context::<DbContext>().ok_or_else(|| ServerFnError::new("DB Context missing"))?;
    
    // Check if setup is already done
    let has_users = db_context.db.has_users().await.unwrap_or(false);
    if has_users {
        return Err(ServerFnError::new("Setup already completed"));
    }

    // Hash password (low cost for MIPS)
    let password_hash = bcrypt::hash(&password, 6)
        .map_err(|_| ServerFnError::new("Hashing error"))?;

    db_context.db.create_user(&username, &password_hash).await
        .map_err(|e| ServerFnError::new(format!("DB error: {}", e)))?;

    Ok(())
}

#[server(Login, "/api/server_fns/Login", encoding = "MessagePack")]
pub async fn login(username: String, password: String) -> Result<UserResponse, ServerFnError> {
    use crate::DbContext;
    use leptos_axum::ResponseOptions;
    use jsonwebtoken::{encode, Header, EncodingKey};
    use cookie::{Cookie, SameSite};
    use std::time::{SystemTime, UNIX_EPOCH};

    let db_context = use_context::<DbContext>().ok_or_else(|| ServerFnError::new("DB Context missing"))?;
    
    let user_opt = db_context.db.get_user_by_username(&username).await
        .map_err(|e| ServerFnError::new(format!("DB error: {}", e)))?;

    if let Some((uid, password_hash)) = user_opt {
        let valid = bcrypt::verify(&password, &password_hash).unwrap_or(false);
        if !valid {
             return Err(ServerFnError::new("Invalid credentials"));
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
            .map_err(|e| ServerFnError::new(format!("Token error: {}", e)))?;

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
        Err(ServerFnError::new("Invalid credentials"))
    }
}

#[server(Logout, "/api/server_fns/Logout", encoding = "MessagePack")]
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

#[server(GetUser, "/api/server_fns/GetUser", encoding = "MessagePack")]
pub async fn get_user() -> Result<Option<UserResponse>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    use jsonwebtoken::{decode, Validation, DecodingKey};

    let headers: HeaderMap = extract().await.map_err(|e| ServerFnError::new(format!("Extract error: {}", e)))?;
    let cookie_header = headers.get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok());

    if let Some(cookie_str) = cookie_header {
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