//! Server-side PostHog capture. Fire-and-forget: a failed analytics call must
//! never slow down or fail a user request.

use crate::config::PosthogConfig;
use serde_json::{Value, json};

#[derive(Clone)]
pub struct Posthog {
    cfg: Option<PosthogConfig>,
    http: reqwest::Client,
    views: std::sync::Arc<std::sync::RwLock<Option<(std::time::Instant, u64)>>>,
}

impl Posthog {
    pub fn new(cfg: Option<PosthogConfig>, http: reqwest::Client) -> Self {
        Self { cfg, http, views: Default::default() }
    }

    pub fn refresh_views(&self, key: String) {
        let analytics = self.clone();
        tokio::spawn(async move {
            let query = "SELECT count() FROM events WHERE event = '$pageview' AND timestamp BETWEEN now() - INTERVAL 24 HOUR AND now() AND properties.$host IN ('builtwithrust.com','www.builtwithrust.com')";
            loop {
                let result: anyhow::Result<u64> = async {
                    let body: Value = analytics
                        .http
                        .post("https://us.posthog.com/api/projects/639107/query/")
                        .bearer_auth(&key)
                        .json(&json!({"query":{"kind":"HogQLQuery","query":query}}))
                        .send()
                        .await?
                        .error_for_status()?
                        .json()
                        .await?;
                    body.pointer("/results/0/0")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| anyhow::anyhow!("invalid aggregate"))
                }
                .await;
                match result {
                    Ok(count) => *analytics.views.write().unwrap() = Some((std::time::Instant::now(), count)),
                    Err(_) => tracing::warn!("could not refresh 24-hour pageview count"),
                }
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
            }
        });
    }

    pub fn recent_views(&self) -> Option<u64> {
        self.views.read().unwrap().filter(|(at, _)| at.elapsed().as_secs() < 900).map(|(_, count)| count)
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
