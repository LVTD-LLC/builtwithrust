//! Runtime configuration, read once from the environment at startup.

use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub site_url: String,
    pub database_url: String,
    pub admin_token: Option<String>,
    pub posthog: Option<PosthogConfig>,
    pub stripe: Option<StripeConfig>,
    pub feature_days: i64,
    pub newsletter: Option<NewsletterConfig>,
    pub sentry_dsn: Option<String>,
    pub sentry_environment: String,
}

#[derive(Clone, Debug)]
pub struct NewsletterConfig {
    pub url: String,
    pub list_uuid: String,
}

#[derive(Clone, Debug)]
pub struct PosthogConfig {
    pub key: String,
    pub host: String,
}

#[derive(Clone, Debug)]
pub struct StripeConfig {
    pub secret_key: String,
    pub webhook_secret: String,
    pub price_id: String,
}

fn opt(name: &str) -> Option<String> {
    env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

impl Config {
    pub fn from_env() -> Self {
        let stripe = match (opt("STRIPE_SECRET_KEY"), opt("STRIPE_WEBHOOK_SECRET"), opt("STRIPE_PRICE_ID")) {
            (Some(secret_key), Some(webhook_secret), Some(price_id)) => {
                Some(StripeConfig { secret_key, webhook_secret, price_id })
            }
            _ => None,
        };
        let posthog = opt("POSTHOG_KEY").map(|key| PosthogConfig {
            key,
            host: opt("POSTHOG_HOST").unwrap_or_else(|| "https://us.i.posthog.com".into()),
        });
        Self {
            bind_addr: opt("BIND_ADDR").unwrap_or_else(|| "0.0.0.0:3000".into()),
            site_url: opt("SITE_URL")
                .unwrap_or_else(|| "http://localhost:3000".into())
                .trim_end_matches('/')
                .to_string(),
            database_url: opt("DATABASE_URL").unwrap_or_else(|| "sqlite://data/builtwithrust.db?mode=rwc".into()),
            admin_token: opt("ADMIN_TOKEN"),
            sentry_dsn: opt("SENTRY_DSN"),
            sentry_environment: opt("SENTRY_ENVIRONMENT").unwrap_or_else(|| "development".into()),
            newsletter: opt("NEWSLETTER_LISTMONK_URL")
                .zip(opt("NEWSLETTER_LIST_UUID"))
                .map(|(url, list_uuid)| NewsletterConfig { url: url.trim_end_matches('/').into(), list_uuid }),
            posthog,
            stripe,
            feature_days: opt("FEATURE_DAYS").and_then(|v| v.parse().ok()).unwrap_or(30),
        }
    }

    pub fn payments_enabled(&self) -> bool {
        self.stripe.is_some()
    }
}
