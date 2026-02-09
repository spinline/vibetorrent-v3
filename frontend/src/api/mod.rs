use gloo_net::http::Request;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Network error")]
    Network,
    #[error("Server error: {status}")]
    Server { status: u16 },
    #[error("Login failed")]
    LoginFailed,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Too many requests")]
    RateLimited,
    #[error("Server function error: {0}")]
    ServerFn(String),
}

fn base_url() -> String {
    "/api".to_string()
}

pub mod auth {
    use super::*;

    #[derive(serde::Serialize)]
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
        pub remember_me: bool,
    }

    pub async fn login(
        username: &str,
        password: &str,
        remember_me: bool,
    ) -> Result<(), ApiError> {
        let req = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
            remember_me,
        };
        let resp = Request::post(&format!("{}/auth/login", base_url()))
            .json(&req)
            .map_err(|_| ApiError::Network)?
            .send()
            .await
            .map_err(|_| ApiError::Network)?;

        if resp.ok() {
            Ok(())
        } else if resp.status() == 429 {
            Err(ApiError::RateLimited)
        } else {
            Err(ApiError::LoginFailed)
        }
    }

    pub async fn logout() -> Result<(), ApiError> {
        Request::post(&format!("{}/auth/logout", base_url()))
            .send()
            .await
            .map_err(|_| ApiError::Network)?;
        Ok(())
    }

    pub async fn check_auth() -> Result<bool, ApiError> {
        let resp = Request::get(&format!("{}/auth/check", base_url()))
            .send()
            .await
            .map_err(|_| ApiError::Network)?;
        Ok(resp.ok())
    }

    #[derive(serde::Deserialize)]
    pub struct UserResponse {
        pub username: String,
    }

    pub async fn get_user() -> Result<UserResponse, ApiError> {
        let resp = Request::get(&format!("{}/auth/check", base_url()))
            .send()
            .await
            .map_err(|_| ApiError::Network)?;
        let user = resp.json().await.map_err(|_| ApiError::Network)?;
        Ok(user)
    }
}

pub mod setup {
    use super::*;

    #[derive(serde::Serialize)]
    pub struct SetupRequest {
        pub username: String,
        pub password: String,
    }

    #[derive(serde::Deserialize)]
    pub struct SetupStatusResponse {
        pub completed: bool,
    }

    pub async fn get_status() -> Result<SetupStatusResponse, ApiError> {
        let resp = Request::get(&format!("{}/setup/status", base_url()))
            .send()
            .await
            .map_err(|_| ApiError::Network)?;
        let status = resp.json().await.map_err(|_| ApiError::Network)?;
        Ok(status)
    }

    pub async fn setup(username: &str, password: &str) -> Result<(), ApiError> {
        let req = SetupRequest {
            username: username.to_string(),
            password: password.to_string(),
        };
        Request::post(&format!("{}/setup", base_url()))
            .json(&req)
            .map_err(|_| ApiError::Network)?
            .send()
            .await
            .map_err(|_| ApiError::Network)?;
        Ok(())
    }
}

pub mod settings {
    use super::*;
    use shared::GlobalLimitRequest;

    pub async fn set_global_limits(req: &GlobalLimitRequest) -> Result<(), ApiError> {
        shared::server_fns::settings::set_global_limits(
            req.max_download_rate,
            req.max_upload_rate,
        )
        .await
        .map_err(|e| ApiError::ServerFn(e.to_string()))
    }
}

pub mod push {
    use super::*;
    use crate::store::PushSubscriptionData;

    pub async fn get_public_key() -> Result<String, ApiError> {
        let resp = Request::get(&format!("{}/push/public-key", base_url()))
            .send()
            .await
            .map_err(|_| ApiError::Network)?;
        let key = resp.text().await.map_err(|_| ApiError::Network)?;
        Ok(key)
    }

    pub async fn subscribe(req: &PushSubscriptionData) -> Result<(), ApiError> {
        Request::post(&format!("{}/push/subscribe", base_url()))
            .json(req)
            .map_err(|_| ApiError::Network)?
            .send()
            .await
            .map_err(|_| ApiError::Network)?;
        Ok(())
    }
}

pub mod torrent {
    use super::*;

    pub async fn add(uri: &str) -> Result<(), ApiError> {
        shared::server_fns::torrent::add_torrent(uri.to_string())
            .await
            .map_err(|e| ApiError::ServerFn(e.to_string()))
    }

    pub async fn action(hash: &str, action: &str) -> Result<(), ApiError> {
        shared::server_fns::torrent::torrent_action(hash.to_string(), action.to_string())
            .await
            .map(|_| ())
            .map_err(|e| ApiError::ServerFn(e.to_string()))
    }

    pub async fn delete(hash: &str) -> Result<(), ApiError> {
        action(hash, "delete").await
    }

    pub async fn delete_with_data(hash: &str) -> Result<(), ApiError> {
        action(hash, "delete_with_data").await
    }

    pub async fn start(hash: &str) -> Result<(), ApiError> {
        action(hash, "start").await
    }

    pub async fn stop(hash: &str) -> Result<(), ApiError> {
        action(hash, "stop").await
    }

    pub async fn set_label(hash: &str, label: &str) -> Result<(), ApiError> {
        shared::server_fns::torrent::set_label(hash.to_string(), label.to_string())
            .await
            .map_err(|e| ApiError::ServerFn(e.to_string()))
    }

    pub async fn set_priority(hash: &str, file_index: u32, priority: u8) -> Result<(), ApiError> {
        shared::server_fns::torrent::set_file_priority(
            hash.to_string(),
            file_index,
            priority,
        )
        .await
        .map_err(|e| ApiError::ServerFn(e.to_string()))
    }
}
