import sqlite3
import unittest
from pathlib import Path


class SubmissionPlacement(unittest.TestCase):
    def setUp(self):
        self.db = sqlite3.connect(":memory:")
        for migration in sorted((Path(__file__).resolve().parents[1] / "migrations").glob("*.sql")):
            if not migration.name.endswith(".down.sql"):
                self.db.executescript(migration.read_text())
        self.db.executescript("""
            INSERT INTO projects(slug,name,tagline,website_url) VALUES ('demo','Demo','Demo','https://example.com');
            INSERT INTO submissions(url,feature_requested) VALUES ('https://example.com',1);
            INSERT INTO payments(stripe_session_id,submission_id) VALUES ('cs_test',1);
        """)

    def check_order(self, pay_first):
        pay = "UPDATE payments SET status='paid', paid_at='2020-01-01T00:00:00Z' WHERE id=1"
        approve = "UPDATE submissions SET status='approved',project_id=1 WHERE id=1"
        first, second = (pay, approve) if pay_first else (approve, pay)
        self.db.execute(first)
        self.assertIsNone(self.db.execute("SELECT featured_until FROM projects").fetchone()[0])
        self.db.execute(second)
        self.db.execute(pay)
        self.db.execute(approve)
        self.assertEqual(self.db.execute("SELECT project_slug FROM newsletter_features").fetchall(), [('demo',)])
        days = self.db.execute("SELECT julianday(featured_until)-julianday('now') FROM projects").fetchone()[0]
        self.assertAlmostEqual(days, 7, places=3)

    def test_payment_before_approval(self):
        self.check_order(True)

    def test_approval_before_payment(self):
        self.check_order(False)

    def test_rejection_does_not_grant_placement(self):
        self.db.execute("UPDATE submissions SET status='rejected'")
        self.db.execute("UPDATE payments SET status='paid'")
        self.assertFalse(self.db.execute("SELECT * FROM newsletter_features").fetchall())
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("UPDATE submissions SET status='approved',project_id=1")
