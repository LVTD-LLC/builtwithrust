//! Server-side PostHog capture. Fire-and-forget: a failed analytics call must
//! never slow down or fail a user request.

use crate::config::PosthogConfig;
use serde_json::{Value, json};

#[derive(Clone)]
pub struct Posthog {
    cfg: Option<PosthogConfig>,
    http: reqwest::Client,
}

impl Posthog {
    pub fn new(cfg: Option<PosthogConfig>, http: reqwest::Client) -> Self {
        Self { cfg, http }
    }

    pub fn enabled(&self) -> bool {
        self.cfg.is_some()
    }

    pub fn config(&self) -> Option<&PosthogConfig> {
        self.cfg.as_ref()
    }

    /// Capture a server-side event. `distinct_id` should be stable per actor
    /// (e.g. a hashed IP for anonymous visitors, or `"server"` for system
    /// events). Properties are merged with `$lib`.
    pub fn capture(&self, event: &str, distinct_id: &str, mut properties: Value) {
        let Some(cfg) = &self.cfg else { return };
        if !properties.is_object() {
            properties = json!({});
        }
        properties["$lib"] = json!("builtwithrust-server");
        let body = json!({
            "api_key": cfg.key,
            "event": event,
            "distinct_id": distinct_id,
            "properties": properties,
            "timestamp": crate::db::now(),
        });
        let url = format!("{}/i/v0/e/", cfg.host.trim_end_matches('/'));
        let http = self.http.clone();
        tokio::spawn(async move {
            if let Err(e) = http.post(url).json(&body).send().await {
                tracing::warn!(error = %e, "posthog capture failed");
            }
        });
    }
}
