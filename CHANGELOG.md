# Changelog

## 2026-10-07

- Add IndexNow ownership verification and automated hourly/post-deploy URL submissions, including updated and removed listings, with retry-safe snapshots. Preserve full sitemap modification timestamps for same-day edits.

- Correct Listmonk public-subscription success parsing (`data.has_optin`) so accepted signups do not show a false delivery error. Cover both new confirmations and existing subscribers.

- Add weekly Rust newsletter signup with Listmonk double opt-in, validation, same-origin protection, bounded signup rate limiting, and recoverable provider errors.
- Keep the homepage cached and add responsive light/dark signup styling; document dedicated Mailgun/Listmonk operations and recovery.

## 2026-10-06

- Show featured projects once in the normal directory, preserving featured-first ordering.
- Improve featured card contrast in light and dark themes with a tinted surface, rust border, clearer secondary text, and solid badge.
