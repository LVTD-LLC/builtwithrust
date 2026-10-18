# Public claim guidance

## Data workflow discovery — verified 2026-10-16

qsv is a command-line toolkit for tabular data; available commands depend on its build/features (https://github.com/dathere/qsv#installation). Its optional `sqlp` command uses Polars, so do not frame qsv and Polars as entirely separate competing engines (https://github.com/dathere/qsv/blob/master/docs/help/sqlp.md). Polars provides DataFrame expressions and lazy queries in Python and Rust (https://docs.pola.rs/user-guide/getting-started/ and https://docs.pola.rs/user-guide/lazy/using/). The Data & AI category owns a short workflow chooser; the existing Python-tools guide owns Polars's place alongside uv and Ruff. No benchmark advantage, universal build support or hands-on results are claimed.

## Warp client licensing and identity — verified 2026-10-15

The official warpdotdev/warp README at revision `53f8a71efb46ea02958802c5a25a9b5b57cfdda3` says its `warpui_core` and `warpui` crates use MIT; the remaining repository code uses AGPL v3. Source: https://github.com/warpdotdev/warp/blob/53f8a71efb46ea02958802c5a25a9b5b57cfdda3/README.md#licensing . The catalog license label must describe this split, not imply proprietary client code or a choice of either license for every file. This does not establish hosted-service pricing or licenses. Warp's terminal/development environment is separate from the Rust HTTP framework documented at https://docs.rs/warp/latest/warp/ . Keep the existing listing URL/title and stored star snapshot.

Verified from source on 2026-10-08: Axum/Maud SSR, SQLite catalog, precompressed page cache, optional Listmonk double opt-in, reviewed project submissions. GitHub stars are catalog snapshots, not live counts. A directory listing does not establish that every component uses Rust. Featured placement is not a quality ranking. Do not invent pricing, source licenses or verification evidence. Re-read src/config.rs, src/routes and AGENTS.md when capabilities change.

## Fish and its editor tooling — verified 2026-10-14

Fish 4.0 ported the shell implementation from C++ to Rust, not every repository asset: its own retrospective describes Fish scripts, Python tools and remaining C. Source: https://fishshell.com/blog/rustport/ . fish-lsp is a separate TypeScript language server for Fish scripts (https://www.fish-lsp.dev/docs/how-it-works); rust-analyzer serves Rust source code (https://rust-analyzer.github.io/). Implementation language does not determine which language server edits a file. The Fish listing owns this short distinction; do not create a duplicate Fish LSP article or label fish-lsp as Rust-built.

## Database and search scope — verified 2026-10-17

Meilisearch's repository LICENSE at revision `541dd6e94c5d6101684610b1af1b50c5aa4e2915` assigns Enterprise Edition code to Business Source License 1.1 and other code to MIT (https://github.com/meilisearch/meilisearch/blob/541dd6e94c5d6101684610b1af1b50c5aa4e2915/LICENSE). Do not imply MIT applies to the entire current repository or to hosted-service plans. SurrealDB's current CLI documentation says FoundationDB support was removed in 3.0.0, while 2.x accepted it (https://surrealdb.com/docs/reference/cli/db/commands/start); deployment options depend on version and edition (https://surrealdb.com/docs/manage/self-hosted/deployment-models). Do not label the old unqualified TiKV/FoundationDB sentence current guidance.

The category owns a short task chooser, not a benchmark comparison: Meilisearch indexes documents for search interfaces (https://www.meilisearch.com/docs/getting_started/overview); SurrealDB combines document/graph storage with integrated full-text search (https://surrealdb.com/docs/learn/data-models/full-text-search/overview). Both support search. Keep existing URLs, titles, catalog star snapshots and unaffected fields.

## Networking task scope — verified 2026-10-18

Official references: [gping](https://github.com/orf/gping) graphs ping times for multiple hosts; [Trippy](https://trippy.rs/) combines traceroute and ping, with platform-specific privilege requirements; [Sniffnet](https://github.com/GyulyVGC/sniffnet) inspects traffic and imports/exports PCAP captures, with platform-specific capture dependencies; [Pingora](https://github.com/cloudflare/pingora) is a Rust framework for programmable network services and HTTP proxies. Do not treat these tools as interchangeable application-health tests or claim a measured performance winner. The infrastructure chooser links only to currently published projects in that category.
