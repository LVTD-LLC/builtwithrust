"""Execute catalog migration transactions: scoped changes, rollback and edit conflicts."""
import json
from pathlib import Path
import sqlite3
import unittest

ROOT = Path(__file__).resolve().parents[1]
OLD = {
    "meilisearch": ("Meilisearch is an open source search engine written in Rust. It offers typo tolerance, filtering, faceting and vector search through a simple REST API, and powers a hosted cloud product.", "MIT"),
    "surrealdb": ("SurrealDB is a multi-model database written in Rust that combines document, graph, relational and time-series workloads behind one query language. It runs embedded, as a single server, or distributed on TiKV or FoundationDB.", "BSL-1.1"),
}
NEW = {p["slug"]: (p["description"], p["license"]) for p in json.loads((ROOT / "seed/projects.json").read_text()) if p["slug"] in OLD}


class DatabaseSearchMigrationTests(unittest.TestCase):
    def setUp(self):
        self.db = sqlite3.connect(":memory:")
        self.db.row_factory = sqlite3.Row
        self.db.executescript((ROOT / "migrations/0001_init.sql").read_text())
        self.addCleanup(self.db.close)

    def insert(self, slug, values):
        self.db.execute("INSERT INTO projects (slug,name,tagline,description,website_url,license,stars,updated_at) VALUES (?, ?, 'Preserve tagline', ?, 'https://example.com', ?, 1234, '2026-10-01T00:00:00Z')", (slug, slug, *values))
        self.db.commit()

    def rows(self):
        return {r["slug"]: dict(r) for r in self.db.execute("SELECT * FROM projects")}

    def migrate(self, direction):
        try:
            self.db.executescript("BEGIN;\n" + (ROOT / f"migrations/0004_database_search_scope.{direction}.sql").read_text() + "\nCOMMIT;")
        except sqlite3.DatabaseError:
            self.db.rollback()
            raise

    def test_scoped_change_and_rollback(self):
        for slug, values in OLD.items():
            self.insert(slug, values)
        self.insert("untouched", OLD["meilisearch"])
        before = self.rows()
        self.migrate("up")
        after = self.rows()
        self.assertEqual(before["untouched"], after["untouched"])
        for slug, values in NEW.items():
            self.assertEqual((after[slug]["description"], after[slug]["license"]), values)
            allowed = {"description", "license", "updated_at"}
            self.assertLessEqual({k for k in before[slug] if before[slug][k] != after[slug][k]}, allowed)
            self.assertNotEqual(before[slug]["updated_at"], after[slug]["updated_at"])
        self.migrate("down")
        for slug, values in OLD.items():
            row = self.rows()[slug]
            self.assertEqual((row["description"], row["license"]), values)

    def test_conflict_aborts_both_rows_in_both_directions(self):
        for direction, source in [("up", OLD), ("down", NEW)]:
            for conflicting in OLD:
                for field in (0, 1):
                    with self.subTest(direction=direction, slug=conflicting, field=field):
                        self.db.execute("DELETE FROM projects")
                        for slug, values in source.items():
                            values = list(values)
                            if slug == conflicting:
                                values[field] = "Intervening editorial change"
                            self.insert(slug, values)
                        before = self.rows()
                        with self.assertRaises(sqlite3.IntegrityError):
                            self.migrate(direction)
                        self.assertEqual(before, self.rows())

    def test_missing_and_already_correct_rows_are_noops(self):
        self.migrate("up")
        self.migrate("down")
        self.assertEqual({}, self.rows())
        for slug, values in NEW.items():
            self.insert(slug, values)
        before = self.rows()
        self.migrate("up")
        self.assertEqual(before, self.rows())


if __name__ == "__main__":
    unittest.main()
