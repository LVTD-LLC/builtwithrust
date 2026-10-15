-- Official source: warpdotdev/warp README Licensing, revision 53f8a71efb46ea02958802c5a25a9b5b57cfdda3 (2026-10-15).
-- Startup runs migrations before constructing the empty page cache.
-- Refuse intervening edits; missing rows and an already-correct seed are safe.
CREATE TEMP TABLE warp_license_guard (ok INTEGER NOT NULL CHECK (ok = 1));
INSERT INTO warp_license_guard (ok)
SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM projects WHERE slug = 'warp'
      AND NOT ((description = 'Warp is a GPU-accelerated terminal written in Rust, with its own UI framework and text rendering. It runs on macOS, Linux and Windows.' AND license IS 'Proprietary')
            OR (description = 'Warp is a GPU-accelerated terminal written in Rust, with its own UI framework and text rendering. It runs on macOS, Linux and Windows. The client repository uses AGPL v3, except for the warpui_core and warpui UI crates, which use MIT. These code licenses do not describe hosted-service terms.' AND license IS 'AGPL-3.0 (client); MIT (UI crates)'))
) THEN 1 ELSE 0 END;
DROP TABLE warp_license_guard;

UPDATE projects
SET description = 'Warp is a GPU-accelerated terminal written in Rust, with its own UI framework and text rendering. It runs on macOS, Linux and Windows. The client repository uses AGPL v3, except for the warpui_core and warpui UI crates, which use MIT. These code licenses do not describe hosted-service terms.',
    license = 'AGPL-3.0 (client); MIT (UI crates)',
    updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
WHERE slug = 'warp' AND description = 'Warp is a GPU-accelerated terminal written in Rust, with its own UI framework and text rendering. It runs on macOS, Linux and Windows.' AND license IS 'Proprietary';
