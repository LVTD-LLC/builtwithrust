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

## License

MIT
