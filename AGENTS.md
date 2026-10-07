# AGENTS.md — operating manual for builtwithrust.com

This codebase is maintained by AI agents. There is no human in the loop on
day-to-day changes, so the rules below are what keep it correct and fast.
Read this whole file before changing anything.

## What this is

A directory of websites, apps and tools built with Rust. Itself built with
Rust: Axum (HTTP), Maud (HTML, compile-time checked), SQLite via sqlx
(compile-time checked SQL), one static binary. Design follows
https://openalternative.co/ (card grid, dotted stat leaders, light/dark).

**Performance is the primary goal.** Developer ergonomics are not. Every
public page is rendered once, pre-compressed (brotli + gzip), and served from
an in-process cache until the catalog changes. Do not add per-request work to
the hot path.

## Layout

```
src/main.rs        CLI: serve (default) | seed [file] | migrate
src/lib.rs         AppState, build() — used by main and tests
src/config.rs      env -> Config (see .env.example for every variable)
src/db.rs          ALL SQL. Models, queries, slugify. Nothing else touches the DB.
src/cache.rs       PageCache: rendered bytes + br/gzip + ETag. purge() on any write.
src/views/         maud templates. mod.rs = layout + shared components.
src/routes/mod.rs  router, AppError, cached_html helper, ClientIp extractor
src/routes/pages.rs  public HTML (cached)
src/routes/api.rs    submit form, Stripe checkout + webhook, healthz, sitemap, robots
src/routes/admin.rs  bearer-token JSON API for agents (this is how listings get in)
src/routes/assets.rs embedded /assets/* (rust-embed), immutable cache headers
src/posthog.rs     server-side capture (fire-and-forget)
src/stripe.rs      checkout session + webhook signature verification (no SDK)
migrations/        sqlx migrations, embedded in the binary, run at startup
assets/            style.css, favicon.svg — embedded, cache-busted by content hash
seed/projects.json initial catalog; `cargo run -- seed`
tests/api.rs       end-to-end tests through the router with a temp SQLite file
scripts/loadtest.sh  oha benchmark; run before and after perf changes
.sqlx/             sqlx offline query metadata. MUST be regenerated when SQL changes.
```

## Invariants (do not break)

1. **Every write purges the cache.** Any handler that changes projects,
   categories, submissions-with-project, or featured state must call
   `state.cache.purge()`. Tests in `tests/api.rs` cover the existing paths;
   add a test for any new write path.
2. **Public pages go through `cached_html`** unless the key space is
   unbounded (search `?q=`) or the response is per-request (form errors,
   404). Never cache anything that depends on cookies or auth.
3. **No external calls on the request path** except: Stripe when creating a
   checkout session (user-initiated) and PostHog capture (spawned, never
   awaited), and Listmonk on explicit newsletter signup (8-second timeout).
   Newsletter GET markup remains cached; never call Listmonk while rendering.
4. **SQL lives in `db.rs`** and uses `sqlx::query!` / `query_as!` where the
   shape is static. The `project_select!` macro builds `&'static str` SQL for
   the shared project+category SELECT; sqlx 0.9 rejects runtime-built strings
   on purpose. Do not use `AssertSqlSafe` to work around that.
5. **Timestamps are RFC3339 UTC strings** (`2026-10-01T12:00:00Z`) so they
   compare correctly as text in SQLite. Use `db::now()`.
6. **Secrets come from env only.** Never commit `.env`. `.env.example` lists
   every variable and must be updated when you add one.
7. **Webhooks are idempotent.** `stripe_events` records every event id; a
   replay is a no-op. Keep it that way.

## Build, test, verify

```bash
cp .env.example .env            # first time; set ADMIN_TOKEN
export DATABASE_URL=sqlite://data/dev.db?mode=rwc
sqlx database create && sqlx migrate run   # or: cargo run -- migrate
cargo run -- seed               # loads seed/projects.json
cargo run                       # http://localhost:3000
cargo test                      # 8 end-to-end tests, ~1s
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

### After changing any SQL or migration

```bash
export DATABASE_URL=sqlite://data/dev.db?mode=rwc
sqlx migrate run
cargo sqlx prepare              # regenerates .sqlx/ — commit it
```

CI builds with `SQLX_OFFLINE=true`, so a stale `.sqlx/` fails the build.
That is intentional.

### Before merging a performance change

```bash
cargo build --release
./target/release/builtwithrust &     # with the seeded dev db
scripts/loadtest.sh                  # prints req/s and p99 for /, a project, a category
```

Compare against the numbers in the last commit that touched
`scripts/BASELINE.md` and update that file if you changed them.

## Admin API (how listings get in)

All routes need `Authorization: Bearer $ADMIN_TOKEN`. JSON in, JSON out.

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/admin/stats` | counts, cache size, which integrations are enabled |
| GET | `/api/admin/projects` | every project, published or not |
| POST | `/api/admin/projects` | create or update one project (keyed by `slug`) |
| POST | `/api/admin/projects/bulk` | array of projects; returns `{upserted, errors}` |
| DELETE | `/api/admin/projects/{slug}` | remove |
| POST | `/api/admin/projects/{slug}/feature` | `{"days": 30}` — feature without payment |
| GET | `/api/admin/submissions?status=pending\|approved\|rejected\|all` | review queue |
| POST | `/api/admin/submissions/{id}/review` | `{"status":"approved","project":{...}}` or `{"status":"rejected"}` |
| POST | `/api/admin/cache/purge` | drop the render cache |

Project payload (only `name`, `tagline`, `website_url` required):

```json
{
  "slug": "zed",
  "name": "Zed",
  "tagline": "High-performance, multiplayer code editor.",
  "description": "Longer text. Blank line = new paragraph.",
  "website_url": "https://zed.dev",
  "repo_url": "https://github.com/zed-industries/zed",
  "logo_url": null,
  "stars": 60000,
  "license": "GPL-3.0",
  "stack": ["gpui", "tokio"],
  "category": "developer-tools",
  "category_name": "Developer Tools",
  "verified": true,
  "published": true
}
```

- `slug` defaults to slugified `name`. Re-posting the same slug updates in place.
- `category` is a slug; it is created if missing, named from `category_name`
  or humanized from the slug.
- `logo_url` null falls back to the site's favicon via DuckDuckGo's icon service.
- `verified: true` shows the gold check. Only set it when you have confirmed
  the site is built with Rust (repo, engineering blog post, job posting).

Scouting workflow for agents:

```bash
# 1. see what people submitted
curl -s -H "Authorization: Bearer $ADMIN_TOKEN" $SITE_URL/api/admin/submissions
# 2. research the URL, decide, then approve (creating the listing) or reject
curl -s -X POST -H "Authorization: Bearer $ADMIN_TOKEN" -H 'content-type: application/json' \
  $SITE_URL/api/admin/submissions/42/review \
  -d '{"status":"approved","project":{"name":"…","tagline":"…","website_url":"…","category":"…","verified":true}}'
# 3. or add discoveries directly
curl -s -X POST -H "Authorization: Bearer $ADMIN_TOKEN" -H 'content-type: application/json' \
  $SITE_URL/api/admin/projects -d @project.json
```

## Payments (Stripe)

Fully wired; disabled until all three env vars are set:
`STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `STRIPE_PRICE_ID`.

- `/feature` shows a form (project + email) → `POST /api/checkout` creates a
  Checkout Session (`mode=payment`, the configured price, metadata
  `project_slug`) and 303s to Stripe.
- Stripe calls `POST /api/stripe/webhook`. Signature is verified with
  HMAC-SHA256 (`stripe.rs`), event ids are deduplicated, and
  `checkout.session.completed` with `payment_status=paid` extends
  `projects.featured_until` by `FEATURE_DAYS` (default 30), then purges cache.
- Success page: `/feature/success?session_id={CHECKOUT_SESSION_ID}`.

To go live: create a one-time Price in Stripe, add a webhook endpoint for
`checkout.session.completed` and `checkout.session.async_payment_succeeded`
pointing at `$SITE_URL/api/stripe/webhook`, paste the three values into env.
`stripe listen --forward-to localhost:3000/api/stripe/webhook` works locally.

## Analytics (PostHog)

Project "Built with Rust" (id 639107, US cloud). `POSTHOG_KEY` is the public
project token; set it and the layout injects the JS snippet (pageviews,
autocapture). Server-side events (`posthog.rs`, `/i/v0/e/`):

| event | when | distinct_id |
|---|---|---|
| `search` | `/?q=` | `server` |
| `submission_created` | public form accepted | hashed IP |
| `submission_reviewed` | admin review | `admin` |
| `project_upserted`, `projects_bulk_upserted` | admin writes | `admin` |
| `checkout_started` | redirect to Stripe | hashed IP |
| `payment_completed` | webhook | `server` |

## Style

- `cargo fmt` (rustfmt.toml: 120 cols) and `cargo clippy -D warnings` must pass.
- Prefer a compile error over a runtime check: typed extractors, maud, `query!`.
- Keep handlers thin; logic that touches data goes in `db.rs`.
- Comments explain *why*. The code already says *what*.
- Dependencies: add one only if it removes more code than it adds. Check
  `cargo tree` for duplicates afterwards.
