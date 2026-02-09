use leptos::prelude::*;

#[server(GetPushPublicKey, "/api/server_fns")]
pub async fn get_public_key() -> Result<String, ServerFnError> {
    let key = std::env::var("VAPID_PUBLIC_KEY")
        .map_err(|_| ServerFnError::new("VAPID_PUBLIC_KEY not configured"))?;
    Ok(key)
}

#[server(SubscribePush, "/api/server_fns")]
pub async fn subscribe_push(
    endpoint: String,
    p256dh: String,
    auth: String,
) -> Result<(), ServerFnError> {
    let db_ctx = expect_context::<crate::DbContext>();
    db_ctx
        .db
        .save_push_subscription(&endpoint, &p256dh, &auth)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to save subscription: {}", e)))
}
