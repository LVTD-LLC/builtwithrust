-- Fish listing accuracy: reviewed source https://fishshell.com/blog/rustport/ (2026-10-14).
-- Migrations run before the app constructs its empty page cache.
-- A missing Fish row is valid on a fresh database. Refuse to overwrite intervening edits.
CREATE TEMP TABLE fish_copy_guard (ok INTEGER NOT NULL CHECK (ok = 1));
INSERT INTO fish_copy_guard (ok)
SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM projects WHERE slug = 'fish'
      AND description NOT IN ('fish is an interactive shell with autosuggestions, syntax highlighting and tab completion that works out of the box. As of version 4.0 the entire codebase was ported from C++ to Rust.', 'fish is an interactive shell with autosuggestions, syntax highlighting and tab completion that works out of the box. In Fish 4.0, the shell implementation was ported from C++ to Rust; supporting scripts and tools use other languages.')
) THEN 1 ELSE 0 END;
DROP TABLE fish_copy_guard;

UPDATE projects
SET description = 'fish is an interactive shell with autosuggestions, syntax highlighting and tab completion that works out of the box. In Fish 4.0, the shell implementation was ported from C++ to Rust; supporting scripts and tools use other languages.',
    updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
WHERE slug = 'fish' AND description = 'fish is an interactive shell with autosuggestions, syntax highlighting and tab completion that works out of the box. As of version 4.0 the entire codebase was ported from C++ to Rust.';
