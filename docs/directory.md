# Directory browsing

The homepage and category pages use `directory::Browse`. All controls submit a
GET form to `/#projects`, so combinations can be bookmarked, shared and used
without JavaScript. Category-route URLs also accept the other controls; the
route category overrides a conflicting query parameter.

- `q`: literal, case-insensitive text in name, tagline, description or stack.
- `category`: category slug; `stack` and `license`: exact case-insensitive values.
- `source`: `available` (repository linked) or `unlisted`. A linked repository
  does **not** assert an open-source license. Unlisted does not mean closed source.
- `verified=1`: existing editorial Rust verification.
- `stars`: `under-1000`, `1000` (at least), `10000` (at least). Unknown counts
  do not qualify; zero is a known count. Stars are catalog snapshots, not live.
- `sort`: empty = recommended (existing featured-first, stars ordering), `stars`,
  `newest` (directory creation, not repository activity), `name`, or `gems`
  (only known counts under 1,000, smallest first). Explicit sorts override
  featured placement but preserve the featured styling; ties use name then slug.

Unknown fixed-choice values fall back to defaults. Unknown category/stack/license
values remain selected and yield no results, instead of silently widening a
shared view. Free-text values are trimmed and limited to 100 characters.

Default homepage/category requests still hit the precompressed HTML cache before
any database access. Arbitrary filter/search combinations are `no-store` and do
not create cache keys. Filtering works over the published catalog returned by
the existing SQL; no schema, SQL metadata, dependencies or external requests
were added. Facet options come only from published records, before filtering,
so users can change a selection without options disappearing. If the catalog
grows substantially, move filtering to checked SQL with pagination.

Verification: `cargo test`, `cargo clippy --all-targets -- -D warnings`,
`cargo fmt --check`; browser checks should cover combined form submission,
reset, back navigation, empty results, 375px/light/dark, and no-JS operation.
