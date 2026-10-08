use builtwithrust::config::Config;
use builtwithrust::db;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "builtwithrust=info".into()))
        .with_target(false)
        .compact()
        .init();

    let cfg = Config::from_env();
    let _sentry = builtwithrust::monitoring::init(&cfg);
    let result = tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(run(cfg));
    if let Err(ref error) = result {
        sentry::integrations::anyhow::capture_anyhow(error);
    }
    result
}

async fn run(cfg: Config) -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("sentry-smoke") => {
            anyhow::ensure!(cfg.sentry_dsn.is_some(), "SENTRY_DSN is required");
            builtwithrust::monitoring::smoke();
            return Ok(());
        }
        Some("seed") => {
            let path = args.next().unwrap_or_else(|| "seed/projects.json".into());
            let pool = db::connect(&cfg.database_url).await?;
            let n = seed(&pool, &path).await?;
            println!("seeded {n} projects from {path}");
            return Ok(());
        }
        Some("migrate") => {
            db::connect(&cfg.database_url).await?;
            println!("migrations applied");
            return Ok(());
        }
        Some(other) => anyhow::bail!("unknown command {other:?}; try `seed [file]` or `migrate`, or no args to serve"),
        None => {}
    }

    if cfg.admin_token.is_none() {
        tracing::warn!("ADMIN_TOKEN is not set: /api/admin/* is disabled");
    }
    if !cfg.payments_enabled() {
        tracing::info!("Stripe keys not set: payments disabled, /feature shows a placeholder");
    }
    if cfg.posthog.is_none() {
        tracing::info!("POSTHOG_KEY not set: analytics disabled");
    }

    let (state, app) = builtwithrust::build(cfg).await?;
    let addr: SocketAddr = state.cfg.bind_addr.parse()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr} (site_url={})", state.cfg.site_url);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn seed(pool: &db::Pool, path: &str) -> anyhow::Result<usize> {
    let raw = std::fs::read_to_string(path)?;
    let inputs: Vec<db::ProjectInput> = serde_json::from_str(&raw)?;
    for input in &inputs {
        db::upsert_project(pool, input).await?;
    }
    Ok(inputs.len())
}

async fn shutdown_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.ok() };
    #[cfg(unix)]
    let terminate =
        async { tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap().recv().await };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<Option<()>>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("shutting down");
}
