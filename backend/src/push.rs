use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;
use web_push::{
    HyperWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};
use futures::StreamExt;

use shared::db::Db;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PushSubscription {
    pub endpoint: String,
    pub keys: PushKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Clone)]
pub struct VapidConfig {
    pub private_key: String,
    pub public_key: String,
    pub email: String,
}

#[derive(Clone)]
pub struct PushSubscriptionStore {
    db: Option<Db>,
    subscriptions: Arc<RwLock<Vec<PushSubscription>>>,
    vapid_config: VapidConfig,
}

impl PushSubscriptionStore {
    pub fn new() -> Self {
        let private_key = std::env::var("VAPID_PRIVATE_KEY").expect("VAPID_PRIVATE_KEY must be set in .env");
        let public_key = std::env::var("VAPID_PUBLIC_KEY").expect("VAPID_PUBLIC_KEY must be set in .env");
        let email = std::env::var("VAPID_EMAIL").expect("VAPID_EMAIL must be set in .env");

        Self {
            db: None,
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            vapid_config: VapidConfig {
                private_key,
                public_key,
                email,
            },
        }
    }

    pub async fn with_db(db: &Db) -> Result<Self, Box<dyn std::error::Error>> {
        let mut subscriptions_vec: Vec<PushSubscription> = Vec::new();

        // Load existing subscriptions from DB
        let subs = db.get_all_push_subscriptions().await?;
        for (endpoint, p256dh, auth) in subs {
            subscriptions_vec.push(PushSubscription {
                endpoint,
                keys: PushKeys { p256dh, auth },
            });
        }
        tracing::info!("Loaded {} push subscriptions from database", subscriptions_vec.len());

        let private_key = std::env::var("VAPID_PRIVATE_KEY").expect("VAPID_PRIVATE_KEY must be set in .env");
        let public_key = std::env::var("VAPID_PUBLIC_KEY").expect("VAPID_PUBLIC_KEY must be set in .env");
        let email = std::env::var("VAPID_EMAIL").expect("VAPID_EMAIL must be set in .env");

        Ok(Self {
            db: Some(db.clone()),
            subscriptions: Arc::new(RwLock::new(subscriptions_vec)),
            vapid_config: VapidConfig {
                private_key,
                public_key,
                email,
            },
        })
    }

    pub async fn add_subscription(&self, subscription: PushSubscription) {
        // Add to memory
        let mut subs = self.subscriptions.write().await;

        // Remove duplicate endpoint if exists
        subs.retain(|s| s.endpoint != subscription.endpoint);
        subs.push(subscription.clone());
        tracing::info!("Added push subscription. Total: {}", subs.len());

        // Save to DB if available
        if let Some(db) = &self.db {
            if let Err(e) = db.save_push_subscription(
                &subscription.endpoint,
                &subscription.keys.p256dh,
                &subscription.keys.auth,
            ).await {
                tracing::error!("Failed to save push subscription to DB: {}", e);
            }
        }
    }

    pub async fn remove_subscription(&self, endpoint: &str) {
        // Remove from memory
        let mut subs = self.subscriptions.write().await;
        subs.retain(|s| s.endpoint != endpoint);
        tracing::info!("Removed push subscription. Total: {}", subs.len());

        // Remove from DB if available
        if let Some(db) = &self.db {
            if let Err(e) = db.remove_push_subscription(endpoint).await {
                tracing::error!("Failed to remove push subscription from DB: {}", e);
            }
        }
    }

    pub async fn get_all_subscriptions(&self) -> Vec<PushSubscription> {
        self.subscriptions.read().await.clone()
    }

    pub fn get_public_key(&self) -> &str {
        &self.vapid_config.public_key
    }
}

/// Send push notification to all subscribed clients
pub async fn send_push_notification(
    store: &PushSubscriptionStore,
    title: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let subscriptions = store.get_all_subscriptions().await;

    if subscriptions.is_empty() {
        tracing::debug!("No push subscriptions to send to");
        return Ok(());
    }

    tracing::info!("Sending push notification to {} subscribers", subscriptions.len());

    let payload = serde_json::json!({
        "title": title,
        "body": body,
        "icon": "/icon-192.png",
        "badge": "/icon-192.png",
        "tag": "vibetorrent"
    });

    let client = Arc::new(HyperWebPushClient::new());
    let vapid_config = store.vapid_config.clone();
    let payload_str = payload.to_string();

    // Send notifications concurrently
    futures::stream::iter(subscriptions)
        .for_each_concurrent(10, |subscription| {
            let client = client.clone();
            let vapid_config = vapid_config.clone();
            let payload_str = payload_str.clone();

            async move {
                let subscription_info = SubscriptionInfo {
                    endpoint: subscription.endpoint.clone(),
                    keys: web_push::SubscriptionKeys {
                        p256dh: subscription.keys.p256dh.clone(),
                        auth: subscription.keys.auth.clone(),
                    },
                };

                let sig_res = VapidSignatureBuilder::from_base64(
                    &vapid_config.private_key,
                    web_push::URL_SAFE_NO_PAD,
                    &subscription_info,
                );

                match sig_res {
                    Ok(mut sig_builder) => {
                        sig_builder.add_claim("sub", vapid_config.email.as_str());
                        sig_builder.add_claim("aud", subscription.endpoint.as_str());
                        
                        match sig_builder.build() {
                            Ok(signature) => {
                                let mut builder = WebPushMessageBuilder::new(&subscription_info);
                                builder.set_vapid_signature(signature);
                                builder.set_payload(web_push::ContentEncoding::Aes128Gcm, payload_str.as_bytes());

                                match builder.build() {
                                    Ok(msg) => {
                                        match client.send(msg).await {
                                            Ok(_) => {
                                                tracing::debug!("Push notification sent to: {}", subscription.endpoint);
                                            }
                                            Err(e) => {
                                                let err_msg = format!("{:?}", e);
                                                tracing::error!("Delivery failed for {}: {}", subscription.endpoint, err_msg);
                                                // Always remove on delivery failure (Gone, Unauthorized, etc.)
                                                tracing::info!("Removing problematic subscription after delivery failure: {}", subscription.endpoint);
                                                let _ = store.remove_subscription(&subscription.endpoint).await;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let err_msg = format!("{:?}", e);
                                        tracing::error!("Encryption/Build failed for {}: {}", subscription.endpoint, err_msg);
                                        // Always remove on encryption failure
                                        tracing::info!("Removing problematic subscription after encryption failure: {}", subscription.endpoint);
                                        let _ = store.remove_subscription(&subscription.endpoint).await;
                                    }
                                }
                            }
                            Err(e) => tracing::error!("Failed to build VAPID signature: {}", e),
                        }
                    }
                    Err(e) => tracing::error!("Failed to create VAPID signature builder: {}", e),
                }
            }
        })
        .await;

        Ok(())

    }

    