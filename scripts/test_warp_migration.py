"""Verify scoped Warp license correction, preservation and transaction rollback."""
import json
from pathlib import Path
import sqlite3
import unittest

ROOT = Path(__file__).resolve().parents[1]
OLD = "Warp is a GPU-accelerated terminal written in Rust, with its own UI framework and text rendering. It runs on macOS, Linux and Windows."
NEW = next(p for p in json.loads((ROOT / "seed/projects.json").read_text()) if p["slug"] == "warp")


class WarpMigrationTests(unittest.TestCase):
    def setUp(self):
        self.db = sqlite3.connect(":memory:")
        self.db.executescript((ROOT / "migrations/0001_init.sql").read_text())
        self.addCleanup(self.db.close)

    def insert(self, description=OLD, license="Proprietary", slug="warp"):
        self.db.execute(
            "INSERT INTO projects (slug,name,tagline,description,website_url,license,stars,updated_at) "
            "VALUES (?, 'Warp', 'Preserve tagline', ?, 'https://www.warp.dev', ?, 24000, '2026-10-01T00:00:00Z')",
            (slug, description, license),
        )
        self.db.commit()

    def migrate(self, direction):
        sql = (ROOT / f"migrations/0003_warp_client_license.{direction}.sql").read_text()
        try:
            self.db.executescript("BEGIN;\n" + sql + "\nCOMMIT;")
        except sqlite3.DatabaseError:
            self.db.rollback()
            raise

    def row(self, slug="warp"):
        self.db.row_factory = sqlite3.Row
        return dict(self.db.execute("SELECT * FROM projects WHERE slug=?", (slug,)).fetchone())

    def test_scoped_change_and_rollback(self):
        self.insert()
        self.insert(slug="other")
        before, other = self.row(), self.row("other")
        self.migrate("up")
        after = self.row()
        self.assertEqual(after["description"], NEW["description"])
        self.assertEqual(after["license"], "AGPL-3.0 (client); MIT (UI crates)")
        self.assertEqual({k for k in before if before[k] != after[k]}, {"description", "license", "updated_at"})
        self.assertEqual(other, self.row("other"))
        self.assertRegex(after["updated_at"], r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
        self.migrate("down")
        self.assertEqual(self.row()["description"], OLD)
        self.assertEqual(self.row()["license"], "Proprietary")
        self.migrate("up")
        self.assertEqual(self.row()["license"], NEW["license"])

    def test_rejects_partial_or_intervening_edits_in_both_directions(self):
        for description, license in [(OLD, "MIT"), ("Edited copy", "Proprietary"), (OLD, None), (NEW["description"], "Proprietary")]:
            with self.subTest(description=description, license=license):
                self.db.execute("DELETE FROM projects")
                self.insert(description, license)
                before = self.row()
                for direction in ("up", "down"):
                    with self.assertRaises(sqlite3.IntegrityError):
                        self.migrate(direction)
                    self.assertEqual(self.row(), before)

    def test_fresh_install_and_current_seed_are_noops(self):
        self.migrate("up")
        self.migrate("down")
        self.assertEqual(self.db.execute("SELECT COUNT(*) FROM projects").fetchone()[0], 0)
        self.insert(NEW["description"], NEW["license"])
        before = self.row()
        self.migrate("up")
        self.assertEqual(self.row(), before)


if __name__ == "__main__":
    unittest.main()
