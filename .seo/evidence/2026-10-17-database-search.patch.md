# Database and search catalog correction

Checked 2026-10-17. Apply through startup migration `migrations/0004_database_search_scope.up.sql`; reverse through its `.down.sql` counterpart. The migration guards all affected rows before either update, permits missing/already-correct rows, and rejects intervening editorial changes. Startup migrations finish before constructing the page cache.

## projects: id 7, slug `meilisearch`

**description before:** Meilisearch is an open source search engine written in Rust. It offers typo tolerance, filtering, faceting and vector search through a simple REST API, and powers a hosted cloud product.

**description after:** Meilisearch is a search engine written in Rust, with typo tolerance, filtering, faceting and vector search through a REST API. Its Community Edition uses MIT; Enterprise Edition code uses Business Source License 1.1. Check the repository license files for the edition and version you plan to deploy. Hosted-service terms are separate.

**license before:** MIT

**license after:** MIT (Community); BUSL-1.1 (Enterprise)

## projects: id 8, slug `surrealdb`

**description before:** SurrealDB is a multi-model database written in Rust that combines document, graph, relational and time-series workloads behind one query language. It runs embedded, as a single server, or distributed on TiKV or FoundationDB.

**description after:** SurrealDB is a multi-model database written in Rust that combines document, graph, relational and time-series workloads behind one query language. It runs embedded or as a server. Storage and distributed-deployment options depend on the version and edition; FoundationDB support was removed in SurrealDB 3.0.0.

Only description/license and `updated_at` change; URLs, names, taglines, star snapshots, created dates, source links and publication flags remain intact. SurrealDB’s existing license label remains unchanged.

Sources: [Meilisearch LICENSE at verified revision](https://github.com/meilisearch/meilisearch/blob/541dd6e94c5d6101684610b1af1b50c5aa4e2915/LICENSE), [SurrealDB current CLI](https://surrealdb.com/docs/reference/cli/db/commands/start), [deployment models](https://surrealdb.com/docs/manage/self-hosted/deployment-models). These are edition/version boundaries, not legal advice or hosted pricing.

Sibling search: authorized read-only project inventory, descriptions containing `FoundationDB` and rows identifying `meilisearch`/`surrealdb`; exactly the two named rows require changes, FoundationDB appears only in SurrealDB. File search covers seed, templates and public claim guidance. Historical migration files and changelog entries remain intact. Underlying catalog observations and full checks remain private in the configured Rowset history.

The canonical Databases & Search category adds a task-specific path into these listings; each listing links back to the chooser. No new URL, request-time SQL/network calls or JavaScript is introduced.
