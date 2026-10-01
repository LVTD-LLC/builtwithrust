//! Minimal Stripe client: Checkout Session creation and webhook signature
//! verification. Talks to the REST API directly so there is no SDK to keep
//! in step with; the two calls used here have been stable for years.

use crate::config::StripeConfig;
use hmac::{Hmac, KeyInit, Mac};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct Stripe {
    cfg: StripeConfig,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutSession {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub data: EventData,
}

#[derive(Debug, Deserialize)]
pub struct EventData {
    pub object: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum StripeError {
    #[error("stripe api error: {0}")]
    Api(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid webhook signature")]
    BadSignature,
    #[error("webhook timestamp outside tolerance")]
    StaleTimestamp,
    #[error("malformed webhook payload: {0}")]
    BadPayload(#[from] serde_json::Error),
}

impl Stripe {
    pub fn new(cfg: StripeConfig, http: reqwest::Client) -> Self {
        Self { cfg, http }
    }

    /// Create a one-time-payment Checkout Session for the configured price.
    /// `project_slug` is stored in metadata and `client_reference_id` so the
    /// webhook can find the project without a database round trip.
    pub async fn create_checkout_session(
        &self,
        project_slug: &str,
        customer_email: Option<&str>,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, StripeError> {
        let mut form: Vec<(&str, String)> = vec![
            ("mode", "payment".into()),
            ("line_items[0][price]", self.cfg.price_id.clone()),
            ("line_items[0][quantity]", "1".into()),
            ("success_url", success_url.into()),
            ("cancel_url", cancel_url.into()),
            ("client_reference_id", project_slug.into()),
            ("metadata[project_slug]", project_slug.into()),
            ("allow_promotion_codes", "true".into()),
        ];
        if let Some(email) = customer_email {
            form.push(("customer_email", email.into()));
        }
        let resp = self
            .http
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(&self.cfg.secret_key, None::<&str>)
            .form(&form)
            .send()
            .await?;
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(StripeError::Api(body));
        }
        Ok(resp.json().await?)
    }

    /// Verify a `Stripe-Signature` header against the raw body and parse the
    /// event. Implements the scheme documented at
    /// https://docs.stripe.com/webhooks#verify-manually
    pub fn parse_webhook(&self, payload: &[u8], sig_header: &str) -> Result<Event, StripeError> {
        let mut timestamp: Option<i64> = None;
        let mut signatures: Vec<&str> = Vec::new();
        for part in sig_header.split(',') {
            let (k, v) = part.trim().split_once('=').ok_or(StripeError::BadSignature)?;
            match k {
                "t" => timestamp = v.parse().ok(),
                "v1" => signatures.push(v),
                _ => {}
            }
        }
        let timestamp = timestamp.ok_or(StripeError::BadSignature)?;
        if signatures.is_empty() {
            return Err(StripeError::BadSignature);
        }
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        if (now - timestamp).abs() > 300 {
            return Err(StripeError::StaleTimestamp);
        }
        let mut mac =
            HmacSha256::new_from_slice(self.cfg.webhook_secret.as_bytes()).expect("hmac accepts any key length");
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(payload);
        let expected = hex::encode(mac.finalize().into_bytes());
        let ok = signatures.iter().any(|s| s.as_bytes().ct_eq(expected.as_bytes()).into());
        if !ok {
            return Err(StripeError::BadSignature);
        }
        Ok(serde_json::from_slice(payload)?)
    }

    /// Build a signature header for a payload. Used by tests and by the
    /// `simulate-webhook` helper to exercise the webhook path locally.
    pub fn sign_for_test(&self, payload: &[u8], timestamp: i64) -> String {
        let mut mac = HmacSha256::new_from_slice(self.cfg.webhook_secret.as_bytes()).unwrap();
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(payload);
        format!("t={timestamp},v1={}", hex::encode(mac.finalize().into_bytes()))
    }
}
