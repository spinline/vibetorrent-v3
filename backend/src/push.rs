use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;
use web_push::{
    IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

// VAPID keys - PRODUCTION'DA ENVIRONMENT VARIABLE'DAN ALINMALI!
const VAPID_PUBLIC_KEY: &str = "BEdPj6XQR7MGzM28Nev9wokF5upHoydNDahouJbQ9ZdBJpEFAN1iNfANSEvY0ItasNY5zcvvqN_tjUt64Rfd0gU";
const VAPID_PRIVATE_KEY: &str = "aUcCYJ7kUd9UClCaWwad0IVgbYJ6svwl19MjSX7GH10";
const VAPID_EMAIL: &str = "mailto:admin@vibetorrent.app";

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

/// In-memory store for push subscriptions
/// TODO: Replace with database in production
#[derive(Default, Clone)]
pub struct PushSubscriptionStore {
    subscriptions: Arc<RwLock<Vec<PushSubscription>>>,
}

impl PushSubscriptionStore {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_subscription(&self, subscription: PushSubscription) {
        let mut subs = self.subscriptions.write().await;
        
        // Remove duplicate endpoint if exists
        subs.retain(|s| s.endpoint != subscription.endpoint);
        
        subs.push(subscription);
        tracing::info!("Added push subscription. Total: {}", subs.len());
    }

    pub async fn remove_subscription(&self, endpoint: &str) {
        let mut subs = self.subscriptions.write().await;
        subs.retain(|s| s.endpoint != endpoint);
        tracing::info!("Removed push subscription. Total: {}", subs.len());
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

    let client = IsahcWebPushClient::new()?;

    for subscription in subscriptions {
        let subscription_info = SubscriptionInfo {
            endpoint: subscription.endpoint.clone(),
            keys: web_push::SubscriptionKeys {
                p256dh: subscription.keys.p256dh.clone(),
                auth: subscription.keys.auth.clone(),
            },
        };

        let mut sig_builder = VapidSignatureBuilder::from_base64(
            VAPID_PRIVATE_KEY,
            web_push::URL_SAFE_NO_PAD,
            &subscription_info,
        )?;
        
        sig_builder.add_claim("sub", VAPID_EMAIL);
        sig_builder.add_claim("aud", subscription.endpoint.clone());
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

pub fn get_vapid_public_key() -> &'static str {
    VAPID_PUBLIC_KEY
}
