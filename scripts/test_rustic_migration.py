"""Guard catalog copy against concurrent edits; verify scoped reversible deployment."""
from pathlib import Path
import sqlite3
import unittest

ROOT = Path(__file__).resolve().parents[1]
OLD = 'rustic is a Rust backup tool that stores encrypted, deduplicated snapshots. It reads and writes the restic repository format, allowing compatible repositories to be used with either tool.\n\nIt is designed for repeatable backup and restore workflows, with support for local and remote storage. Its command-line application builds on separate Rust libraries for backup operations and storage backends.'
NEW = "rustic is a Rust backup tool that stores encrypted, deduplicated snapshots. It reads and writes the restic repository format, but sharing a format does not make concurrent mixed-client operations safe. Do not run restic prune while rustic may be writing to the same repository: rustic does not honor restic's locks, and that overlap can leave the repository inconsistent.\n\nIt is designed for repeatable backup and restore workflows, with support for local and remote storage. Its command-line application builds on separate Rust libraries for backup operations and storage backends."


class RusticMigrationTests(unittest.TestCase):
    def setUp(self):
        self.db = sqlite3.connect(":memory:")
        self.db.row_factory = sqlite3.Row
        self.db.executescript((ROOT / "migrations/0001_init.sql").read_text())
        self.addCleanup(self.db.close)

    def insert(self, description, slug="rustic"):
        self.db.execute(
            "INSERT INTO projects (slug,name,tagline,description,website_url,license,stars,updated_at) "
            "VALUES (?, 'rustic', 'Preserve tagline', ?, 'https://rustic.cli.rs', 'Apache-2.0 OR MIT', 3300, '2000-01-01T00:00:00Z')",
            (slug, description),
        )
        self.db.commit()

    def rows(self):
        return {r["slug"]: dict(r) for r in self.db.execute("SELECT * FROM projects")}

    def migrate(self, direction):
        sql = (ROOT / f"migrations/0005_rustic_repository_safety.{direction}.sql").read_text()
        try:
            self.db.executescript("BEGIN;\n" + sql + "\nCOMMIT;")
        except sqlite3.DatabaseError:
            self.db.rollback()
            raise

    def test_scoped_copy_and_reversal(self):
        self.insert(OLD)
        self.insert(OLD, "unrelated")
        before = self.rows()
        self.migrate("up")
        after = self.rows()
        self.assertEqual(after["rustic"]["description"], NEW)
        self.assertEqual(after["unrelated"], before["unrelated"])
        self.assertEqual({k for k in before["rustic"] if before["rustic"][k] != after["rustic"][k]}, {"description", "updated_at"})
        self.assertRegex(after["rustic"]["updated_at"], r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
        self.migrate("down")
        self.assertEqual(self.rows()["rustic"]["description"], OLD)

    def test_editorial_conflict_aborts_in_both_directions(self):
        self.insert("An intervening catalog edit")
        before = self.rows()
        for direction in ("up", "down"):
            with self.subTest(direction=direction):
                with self.assertRaises(sqlite3.IntegrityError):
                    self.migrate(direction)
                self.assertEqual(self.rows(), before)

    def test_missing_and_already_applied_are_noops(self):
        for direction in ("up", "down"):
            self.migrate(direction)
        self.assertEqual(self.rows(), {})
        for direction, description in (("up", NEW), ("down", OLD)):
            with self.subTest(direction=direction):
                self.db.execute("DELETE FROM projects")
                self.insert(description)
                before = self.rows()
                self.migrate(direction)
                self.assertEqual(self.rows(), before)


if __name__ == "__main__":
    unittest.main()
