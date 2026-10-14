"""Exercise the reviewed catalog correction on real SQLite, including edit conflicts."""
import json
from pathlib import Path
import sqlite3
import unittest

ROOT = Path(__file__).resolve().parents[1]
OLD = (
    "fish is an interactive shell with autosuggestions, syntax highlighting and tab "
    "completion that works out of the box. As of version 4.0 the entire codebase "
    "was ported from C++ to Rust."
)
NEW = next(p["description"] for p in json.loads((ROOT / "seed/projects.json").read_text()) if p["slug"] == "fish")


class FishMigrationTests(unittest.TestCase):
    def setUp(self):
        self.db = sqlite3.connect(":memory:")
        self.db.executescript((ROOT / "migrations/0001_init.sql").read_text())
        self.addCleanup(self.db.close)

    def insert(self, description=OLD, slug="fish"):
        self.db.execute(
            "INSERT INTO projects (slug,name,tagline,description,website_url,stars,updated_at) "
            "VALUES (?, 'Fish', 'Keep this tagline', ?, 'https://fishshell.com', 30000, '2026-10-01T00:00:00Z')",
            (slug, description),
        )
        self.db.commit()

    def migrate(self, direction):
        sql = (ROOT / f"migrations/0002_fish_implementation.{direction}.sql").read_text()
        # SQLx runs each migration in a transaction; reproduce rollback on a failed guard.
        try:
            self.db.executescript("BEGIN;\n" + sql + "\nCOMMIT;")
        except sqlite3.DatabaseError:
            self.db.rollback()
            raise

    def test_corrects_only_expected_row_and_reverses(self):
        self.insert()
        self.insert("Another project", slug="other")
        self.migrate("up")
        row = self.db.execute("SELECT description,tagline,stars,updated_at FROM projects WHERE slug='fish'").fetchone()
        self.assertEqual(row[:3], (NEW, "Keep this tagline", 30000))
        self.assertGreater(row[3], "2026-10-01T00:00:00Z")
        self.assertRegex(row[3], r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
        self.assertEqual(self.db.execute("SELECT description FROM projects WHERE slug='other'").fetchone()[0], "Another project")
        self.migrate("down")
        self.assertEqual(self.db.execute("SELECT description FROM projects WHERE slug='fish'").fetchone()[0], OLD)
        self.migrate("up")
        self.assertEqual(self.db.execute("SELECT description FROM projects WHERE slug='fish'").fetchone()[0], NEW)

    def test_intervening_edit_is_not_overwritten_in_either_direction(self):
        self.insert("An editor changed this")
        for direction in ("up", "down"):
            with self.assertRaises(sqlite3.IntegrityError):
                self.migrate(direction)
            self.assertEqual(self.db.execute("SELECT description FROM projects").fetchone()[0], "An editor changed this")

    def test_fresh_install_and_already_correct_seed_are_safe(self):
        self.migrate("up")
        self.assertEqual(self.db.execute("SELECT COUNT(*) FROM projects").fetchone()[0], 0)
        self.insert(NEW)
        self.migrate("up")
        self.assertEqual(self.db.execute("SELECT description,updated_at FROM projects").fetchone(), (NEW, "2026-10-01T00:00:00Z"))


if __name__ == "__main__":
    unittest.main()
