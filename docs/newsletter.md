# Weekly Rust newsletter

## Services and credentials

- Admin: https://newsletter.builtwithrust.com/admin/ (username `rasul`).
- Main host `138.201.126.181`, CapRover `https://captain.cr.lvtd.dev`.
- Apps/services: `builtwithrust-listmonk`, `builtwithrust-listmonk-db` (no prefix).
- Listmonk 6.2.0, upstream image `listmonk/listmonk@sha256:f535d59e14991337a9f2d570273685378ae86b0d7698c3e00da444e3bc205286`.
- PostgreSQL 17 (`postgres:17-alpine`), no published ports. Volumes:
  `builtwithrust-listmonk-db-data` at `/var/lib/postgresql/data`,
  `builtwithrust-listmonk-uploads` at `/listmonk/uploads`.
- Infisical: **Openclaw / prod / /projects/builtwithrust**, keys
  `LISTMONK_ADMIN_USER`, `LISTMONK_ADMIN_PASSWORD`, `LISTMONK_DB_PASSWORD`,
  `LISTMONK_SMTP_PASSWORD`, `LISTMONK_URL`, `NEWSLETTER_LIST_UUID`.
  Never put credentials in source, logs or chat. Bootstrap admin env removed.
- Startup uses idempotent installation then normal startup, not automatic upgrades.

## Email and consent

Sender: **Rasul at Built with Rust <rasul@builtwithrust.com>**.
Mailgun US `mg.builtwithrust.com`, SMTP `smtp.mailgun.org:587`, STARTTLS with
certificate verification; login `postmaster@mg.builtwithrust.com`.
SPF, 2048-bit DKIM, MX and tracking CNAME are DNS-only; relaxed, monitor-only
DMARC on apex and sending subdomain. Existing apex records remain unchanged.
This is outbound email, not a receiving inbox for the sender address.

Only public list: **Weekly Rust news and projects**, double opt-in. The site
calls the public subscription endpoint with the fixed list UUID; it cannot
preconfirm, override suppressions, or access Listmonk admin APIs. Existing
subscribers get the same generic success screen. Confirmation and unsubscribe
are served by Listmonk. Open/click tracking is disabled in both systems.
Mailgun handles bounce/complaint suppressions; no webhook mirrors these into
Listmonk. Include its unsubscribe footer in every campaign.
No editorial campaign or automatic weekly sending is created by this deployment.

## Website

`NEWSLETTER_LISTMONK_URL=http://builtwithrust-listmonk:9000` and
`NEWSLETTER_LIST_UUID` enable the homepage form. Missing configuration hides it.
`POST /newsletter` requires a matching Origin header, validates email, ignores
honeypot submissions, limits attempts to 10/IP/hour, and uses an 8-second
Listmonk timeout. Failure returns a no-store 503 with a retryable form. Success
redirects to a generic no-store inbox screen. Emails/provider bodies are not
logged or stored in the directory database. No schema/catalog changes.

The limiter is bounded to 10,000 active keys, lives in memory, and resets on
restart. Production has one replica. Move it to shared storage or the edge
before scaling. CapRover must overwrite `X-Real-IP`, and the application port
must remain private; do not trust visitor-controlled forwarded-IP headers.

Homepage HTML is still rendered/pre-compressed once and cached; no Listmonk
request occurs on GET. Signup is a user-initiated request-path exception.

The Listmonk custom Nginx template blocks external `/api/public/subscription`
and redirects `/subscription`, `/subscription/` and `/subscription/form*` to
https://builtwithrust.com/newsletter. This prevents bypassing the site limiter.
UUID confirmation, preference and unsubscribe routes remain accessible.

## Backups and rollback

The existing nightly `restic-main` job discovers PostgreSQL and includes Docker
volumes. Initial dump: `/docker/data/builtwithrust-listmonk-backups/initial-2026-10-07.dump`.
Before upgrades take a fresh consistent `pg_dump -Fc` plus uploads backup and
restore to an isolated database. Do not downgrade after schema changes without
restoring the matching backup. Keep each instance in the control-plane inventory.

To disable website signup clear `NEWSLETTER_LIST_UUID`, or revert the feature
PR. Preserve the Listmonk database and uploads; never delete subscriber data as
part of website rollback. Before sending the first edition, preview/test it and
verify the sender, target list, unsubscribe footer and suppression handling.

## Verification

Local tests cover disabled config, cached homepage, origin/validation rejection,
fixed-list payload, duplicate generic success, honeypot, provider failures and
rate limiting. Preview browser checks passed at 1440/390 in light/dark with no
overflow, native email validation, and a deliberate provider outage returning
an accessible 503 form. CI runs fmt, clippy, tests and release build.
