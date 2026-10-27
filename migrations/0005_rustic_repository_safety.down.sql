-- Source: https://rustic.cli.rs/docs/FAQ.html#can-i-use-rustic-with-my-existing-restic-repositories
-- Verified 2026-10-27. Startup migrates before constructing the empty page cache.
-- Refuse intervening copy edits; missing and already-applied rows are no-ops.
CREATE TEMP TABLE rustic_copy_guard (ok INTEGER NOT NULL CHECK (ok = 1));
INSERT INTO rustic_copy_guard (ok)
SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM projects WHERE slug = 'rustic'
      AND NOT (description = 'rustic is a Rust backup tool that stores encrypted, deduplicated snapshots. It reads and writes the restic repository format, but sharing a format does not make concurrent mixed-client operations safe. Do not run restic prune while rustic may be writing to the same repository: rustic does not honor restic''s locks, and that overlap can leave the repository inconsistent.

It is designed for repeatable backup and restore workflows, with support for local and remote storage. Its command-line application builds on separate Rust libraries for backup operations and storage backends.' OR description = 'rustic is a Rust backup tool that stores encrypted, deduplicated snapshots. It reads and writes the restic repository format, allowing compatible repositories to be used with either tool.

It is designed for repeatable backup and restore workflows, with support for local and remote storage. Its command-line application builds on separate Rust libraries for backup operations and storage backends.')
) THEN 1 ELSE 0 END;
DROP TABLE rustic_copy_guard;

UPDATE projects
SET description = 'rustic is a Rust backup tool that stores encrypted, deduplicated snapshots. It reads and writes the restic repository format, allowing compatible repositories to be used with either tool.

It is designed for repeatable backup and restore workflows, with support for local and remote storage. Its command-line application builds on separate Rust libraries for backup operations and storage backends.',
    updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
WHERE slug = 'rustic' AND description = 'rustic is a Rust backup tool that stores encrypted, deduplicated snapshots. It reads and writes the restic repository format, but sharing a format does not make concurrent mixed-client operations safe. Do not run restic prune while rustic may be writing to the same repository: rustic does not honor restic''s locks, and that overlap can leave the repository inconsistent.

It is designed for repeatable backup and restore workflows, with support for local and remote storage. Its command-line application builds on separate Rust libraries for backup operations and storage backends.';
