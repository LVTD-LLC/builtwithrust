# builtwithrust.com

A directory of websites, apps and tools built with Rust. Itself built with Rust.

- **Axum** for HTTP, **Maud** for compile-time-checked HTML, **SQLite** via
  **sqlx** with compile-time-checked SQL.
- Every public page is rendered once, pre-compressed (brotli/gzip) and served
  from memory with an ETag. Any catalog write purges the cache.
- One static binary, embedded assets and migrations. Nothing to install next
  to it besides the SQLite file (and optionally Litestream for backups).
- Public submission form with review queue, bearer-token admin JSON API for
  agents, Stripe-powered "featured" slots, PostHog analytics.

## Run it

```bash
cp .env.example .env            # set ADMIN_TOKEN at least
export DATABASE_URL=sqlite://data/dev.db?mode=rwc
cargo run -- migrate
cargo run -- seed               # loads seed/projects.json
cargo run                       # http://localhost:3000
```

Tests: `cargo test`. Load test: `scripts/loadtest.sh` (needs [oha](https://github.com/hatoo/oha)).

Everything else, including the admin API reference, Stripe and PostHog setup
and the rules for changing the code, is in [AGENTS.md](AGENTS.md).

## Deployment

Production runs on CapRover at https://builtwithrust.com. Pushes to `main`
run the Rust checks, build an image tagged with the commit SHA in
`ghcr.io/lvtd-llc/builtwithrust`, and deploy it using the app's deployment token.
The workflow can also be run manually from `main`.
It waits until `/healthz` reports the deployed commit in its
`X-Deployment-Revision` header before marking the rollout successful.

GitHub Actions secrets: `CAPROVER_SERVER` (dashboard origin) and `APP_TOKEN`
(the `builtwithrust` app token). Image publishing uses `GITHUB_TOKEN`.
CapRover must have permission to pull the image from GHCR.

CapRover settings:

- Container HTTP port: `3000`; one instance pinned to its storage node.
- Persistent volume `builtwithrust-data` mounted at `/data`.
- `SITE_URL=https://builtwithrust.com` and a generated `ADMIN_TOKEN`.
- `DATABASE_URL=sqlite:///data/builtwithrust.db?mode=rwc` (the image default).
- PostHog settings from `.env.example`; Stripe stays disabled until configured.
- Custom domain `builtwithrust.com`, HTTPS enabled, and force HTTPS enabled.

Migrations run automatically at startup. Seed a fresh deployment once with
`builtwithrust seed` inside the running container, then purge the page cache
through the admin API. Never seed automatically on deploy: it would overwrite
catalog edits. The runtime uses UID `10001`, which must own `/data`.

To roll back, deploy a previous commit's image tag through CapRover. The
SQLite volume survives deployments; schema migrations may require a separate
database recovery. Persistent storage is not a backup; `litestream.yml`
provides a starting point for configuring backups separately.

## License

MIT
