//! builtwithrust — a directory of things built with Rust, itself built with Rust.
//!
//! Module map:
//! - [`config`]  env → `Config`
//! - [`db`]      SQLite models and every SQL statement
//! - [`cache`]   rendered-page cache (the hot path)
//! - [`views`]   maud templates
//! - [`routes`]  axum handlers, grouped by audience (public / api / admin)
//! - [`posthog`] server-side analytics
//! - [`stripe`]  checkout + webhook verification

pub mod cache;
pub mod config;
pub mod db;
pub mod directory;
pub mod newsletter;
pub mod posthog;
pub mod routes;
pub mod stripe;
pub mod views;

use crate::cache::PageCache;
use crate::config::Config;
use crate::posthog::Posthog;
use crate::stripe::Stripe;
use rust_embed::Embed;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Embed)]
#[folder = "assets/"]
pub struct Assets;

pub struct AppState {
    pub cfg: Config,
    pub pool: db::Pool,
    pub cache: PageCache,
    pub posthog: Posthog,
    pub stripe: Option<Stripe>,
    pub http: reqwest::Client,
    /// Short hash of all embedded assets, used to cache-bust asset URLs.
    pub asset_version: String,
    pub signup_limiter: newsletter::SignupLimiter,
}

impl AppState {
    pub fn asset_url(&self, name: &str) -> String {
        format!("/assets/{name}?v={}", self.asset_version)
    }
}

pub type SharedState = Arc<AppState>;

/// Build the application state and router from a config. Used by `main` and
/// by the integration tests.
pub async fn build(cfg: Config) -> anyhow::Result<(SharedState, axum::Router)> {
    let pool = db::connect(&cfg.database_url).await?;
    let http = reqwest::Client::builder()
        .user_agent(concat!("builtwithrust/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let posthog = Posthog::new(cfg.posthog.clone(), http.clone());
    let stripe = cfg.stripe.clone().map(|s| Stripe::new(s, http.clone()));
    let asset_version = {
        let mut h = Sha256::new();
        for name in Assets::iter() {
            h.update(name.as_bytes());
            if let Some(f) = Assets::get(&name) {
                h.update(&f.data);
            }
        }
        hex::encode(&h.finalize()[..6])
    };
    let state = Arc::new(AppState {
        cfg,
        pool,
        cache: PageCache::default(),
        posthog,
        stripe,
        http,
        asset_version,
        signup_limiter: newsletter::SignupLimiter::default(),
    });
    let router = routes::router(state.clone());
    Ok((state, router))
}
