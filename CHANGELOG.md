# Changelog

## 2026-10-08

- Add combinable search, category, crate/stack, license, repository, verification, and star-count filters with shareable URLs and no JavaScript requirement.
- Add most-starred, recently-added, alphabetical, and hidden-gems views; retain featured-first recommended ordering and existing featured-card contrast.
- Add responsive controls, selected-state persistence, result counts and reset guidance; preserve cached default pages and bypass cache for arbitrary filter combinations.

## 2026-10-07

- Add IndexNow ownership verification and automated hourly/post-deploy URL submissions, including updated and removed listings, with retry-safe snapshots. Preserve full sitemap modification timestamps for same-day edits.

- Correct Listmonk public-subscription success parsing (`data.has_optin`) so accepted signups do not show a false delivery error. Cover both new confirmations and existing subscribers.

- Add weekly Rust newsletter signup with Listmonk double opt-in, validation, same-origin protection, bounded signup rate limiting, and recoverable provider errors.
- Keep the homepage cached and add responsive light/dark signup styling; document dedicated Mailgun/Listmonk operations and recovery.

## 2026-10-06

- Show featured projects once in the normal directory, preserving featured-first ordering.
- Improve featured card contrast in light and dark themes with a tinted surface, rust border, clearer secondary text, and solid badge.
