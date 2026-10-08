# Sentry monitoring

Dedicated project: [Built with Rust](https://rasulkireev.sentry.io/projects/builtwithrust/).
Rust SDK 0.49.3; browser SDK 11.5.0. The readable browser bundle is self-hosted,
content-hash cache-busted and reproducibly built with `npm ci --include=dev &&
npm run build:monitoring`. Commit the generated asset; CI checks it matches source.

## Configuration and coverage

- `SENTRY_DSN`: public ingest DSN, optional. Unset disables both SDKs.
- `SENTRY_ENVIRONMENT`: set to `production` in the deployed service.
- Release is the existing `BUILD_REVISION` baked into the image.
- Errors: Rust handler failures and startup errors, plus unhandled browser errors.
  Rust's panic hook is installed, but release builds abort on panic: delivery on
  fatal process termination is best-effort, not guaranteed. No crash-dump collection. The release retains symbols and app line tables for
  locally resolved error frames; the image enables `RUST_LIB_BACKTRACE=1`
  for origin stacks and the SDK attaches a fallback capture stack when needed; no source upload or management token in CI.
- Tracing: 20% of server requests and browser page loads/operations. Server routes
  use matched templates (never raw paths); health/static/robots/sitemap are excluded.
  Incoming `sentry-trace` is supported; untrusted baggage is not collected. Browser
  propagation is restricted to same-origin API/newsletter calls. Cached HTML does
  not contain per-request trace IDs, so initial browser/server page-load traces are
  intentionally separate. These are request-level traces, not automatic SQL spans.
- Application metrics: `http.server.requests` (count), `http.server.duration`
  (milliseconds), `browser.page_loaded` (count). Bounded route/status attributes;
  no search values, project names, emails or IDs. Backend metrics are not trace-sampled.
- Logs: structured HTTP completion records for sampled requests and all server
  failures; browser page-loaded records. Existing free-text application/third-party
  tracing logs stay local rather than forwarding potentially sensitive content.
- Session replay: 10% of eligible public catalog page sessions, 100% on error.
  All text/inputs masked and media/scripts blocked. No network body recording;
  custom recording events (including console/network metadata) dropped. No replay
  on query/fragment URLs, forms, newsletter, payment, privacy or unknown pages.
- Profiling: browser UI profiling, 20% of sessions, trace lifecycle, with
  `Document-Policy: js-profiling`. Chromium only, restricted to the same query-free public catalog pages as replay. **Sentry does not support native
  Rust CPU profiling**; this integration must not be described as backend profiling.

Browser span scrubbing handles the SDK v11 streamed `name`/`attributes` format
and envelope dynamic-sampling metadata; a real-SDK transport regression checks
serialized envelopes and preserves profile linkage and release/environment.

Both SDKs disable default PII and remove free-text exception values, requests,
user data, extras and breadcrumbs from error events. Backend transport batches on
its own thread, with no Sentry network round trip in a request. `/privacy` discloses
monitoring. Do not add personal data through new tags or custom telemetry.

## Verification and rollback

Run `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
`npm test`, and the reproducible bundle check. Tests cover concurrent hub isolation,
secret scrubbing, and replay exclusions.

Inside the deployed container, `builtwithrust sentry-smoke` emits one explicitly
operator-triggered synthetic error/log/metric without altering the database or
exposing a public debug route. Record its event ID and confirm ingestion in Sentry.
Verify real browser envelopes and Sentry readback separately: HTTP acceptance alone
is not proof of stored/replayable data. Resolve synthetic issues after verification.

Rollback by removing only `SENTRY_DSN` from the full existing CapRover definition
(or deploying the preceding immutable image). Preserve all unrelated environment,
replica, network, domain and security fields. Check rollout and `/healthz` revision.
No Sentry API token is needed in the application. Keep management credentials out
of CapRover app env and Git.

References: [Rust](https://docs.sentry.io/platforms/rust/),
[metrics](https://docs.sentry.io/platforms/rust/metrics/),
[logs](https://docs.sentry.io/platforms/rust/logs/),
[browser profiling](https://docs.sentry.io/platforms/javascript/profiling/).
