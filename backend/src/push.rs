use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;
use web_push::{
    HyperWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

use crate::db::Db;

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
pub struct PushSubscriptionStore {
    db: Option<Db>,
    subscriptions: Arc<RwLock<Vec<PushSubscription>>>,
}

impl PushSubscriptionStore {
    pub fn new() -> Self {
        Self {
            db: None,
            subscriptions: Arc::new(RwLock::new(Vec::new())),
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

        Ok(Self {
            db: Some(db.clone()),
            subscriptions: Arc::new(RwLock::new(subscriptions_vec)),
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

    let client = HyperWebPushClient::new();

    let vapid_private_key = std::env::var("VAPID_PRIVATE_KEY").expect("VAPID_PRIVATE_KEY must be set in .env");
    let vapid_email = std::env::var("VAPID_EMAIL").expect("VAPID_EMAIL must be set in .env");

    for subscription in subscriptions {
        let subscription_info = SubscriptionInfo {
            endpoint: subscription.endpoint.clone(),
            keys: web_push::SubscriptionKeys {
                p256dh: subscription.keys.p256dh.clone(),
                auth: subscription.keys.auth.clone(),
            },
        };

        let mut sig_builder = VapidSignatureBuilder::from_base64(
            &vapid_private_key,
            web_push::URL_SAFE_NO_PAD,
            &subscription_info,
        )?;

        sig_builder.add_claim("sub", vapid_email.as_str());
        sig_builder.add_claim("aud", subscription.endpoint.as_str());
        let signature = sig_builder.build()?;

        let mut builder = WebPushMessageBuilder::new(&subscription_info);
        builder.set_vapid_signature(signature);

        let payload_str = payload.to_string();
        builder.set_payload(web_push::ContentEncoding::Aes128Gcm, payload_str.as_bytes());

        match client.send(builder.build()?).await {
            Ok(_) => {
                tracing::debug!("Push notification sent to: {}", subscription.endpoint);
            }
            Err(e) => {
                tracing::error!("Failed to send push notification: {}", e);
                // TODO: Remove invalid subscriptions
            }
        }
    }

    Ok(())
}

pub fn get_vapid_public_key() -> String {
    std::env::var("VAPID_PUBLIC_KEY").expect("VAPID_PUBLIC_KEY must be set in .env")
}
