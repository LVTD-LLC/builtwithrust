-- Sources checked 2026-10-17: Meilisearch LICENSE at 541dd6e94c5d6101684610b1af1b50c5aa4e2915;
-- https://surrealdb.com/docs/reference/cli/db/commands/start (3.x, FoundationDB removal).
-- Startup migrates before creating the page cache. Reject intervening edits atomically.
CREATE TEMP TABLE catalog_scope_guard (ok INTEGER NOT NULL CHECK (ok = 1));
INSERT INTO catalog_scope_guard (ok) SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM projects WHERE slug = 'meilisearch'
    AND NOT ((description = 'Meilisearch is an open source search engine written in Rust. It offers typo tolerance, filtering, faceting and vector search through a simple REST API, and powers a hosted cloud product.' AND license IS 'MIT') OR (description = 'Meilisearch is a search engine written in Rust, with typo tolerance, filtering, faceting and vector search through a REST API. Its Community Edition uses MIT; Enterprise Edition code uses Business Source License 1.1. Check the repository license files for the edition and version you plan to deploy. Hosted-service terms are separate.' AND license IS 'MIT (Community); BUSL-1.1 (Enterprise)'))
) THEN 1 ELSE 0 END;
INSERT INTO catalog_scope_guard (ok) SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM projects WHERE slug = 'surrealdb'
    AND NOT ((description = 'SurrealDB is a multi-model database written in Rust that combines document, graph, relational and time-series workloads behind one query language. It runs embedded, as a single server, or distributed on TiKV or FoundationDB.' AND license IS 'BSL-1.1') OR (description = 'SurrealDB is a multi-model database written in Rust that combines document, graph, relational and time-series workloads behind one query language. It runs embedded or as a server. Storage and distributed-deployment options depend on the version and edition; FoundationDB support was removed in SurrealDB 3.0.0.' AND license IS 'BSL-1.1'))
) THEN 1 ELSE 0 END;
DROP TABLE catalog_scope_guard;

UPDATE projects
SET description = 'Meilisearch is a search engine written in Rust, with typo tolerance, filtering, faceting and vector search through a REST API. Its Community Edition uses MIT; Enterprise Edition code uses Business Source License 1.1. Check the repository license files for the edition and version you plan to deploy. Hosted-service terms are separate.',
    license = 'MIT (Community); BUSL-1.1 (Enterprise)',
    updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
WHERE slug = 'meilisearch' AND description = 'Meilisearch is an open source search engine written in Rust. It offers typo tolerance, filtering, faceting and vector search through a simple REST API, and powers a hosted cloud product.' AND license IS 'MIT';

UPDATE projects
SET description = 'SurrealDB is a multi-model database written in Rust that combines document, graph, relational and time-series workloads behind one query language. It runs embedded or as a server. Storage and distributed-deployment options depend on the version and edition; FoundationDB support was removed in SurrealDB 3.0.0.',
    license = 'BSL-1.1',
    updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
WHERE slug = 'surrealdb' AND description = 'SurrealDB is a multi-model database written in Rust that combines document, graph, relational and time-series workloads behind one query language. It runs embedded, as a single server, or distributed on TiKV or FoundationDB.' AND license IS 'BSL-1.1';

