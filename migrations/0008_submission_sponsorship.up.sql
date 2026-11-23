ALTER TABLE submissions ADD COLUMN repo_url TEXT;
ALTER TABLE submissions ADD COLUMN twitter_url TEXT;
ALTER TABLE submissions ADD COLUMN feature_requested INTEGER NOT NULL DEFAULT 0;
CREATE INDEX approved_project_social ON submissions(project_id,id) WHERE status='approved' AND twitter_url IS NOT NULL;
CREATE TRIGGER guard_sponsored_review BEFORE UPDATE OF status,project_id ON submissions
WHEN NEW.feature_requested AND (
    (OLD.status!='pending' AND (OLD.status!=NEW.status OR OLD.project_id IS NOT NEW.project_id))
    OR (NEW.status='approved' AND NOT EXISTS(SELECT 1 FROM projects WHERE id=NEW.project_id AND published=1)))
BEGIN SELECT RAISE(ABORT, 'Sponsored approval requires a published project and a consistent review decision'); END;
ALTER TABLE payments ADD COLUMN submission_id INTEGER REFERENCES submissions(id);
CREATE UNIQUE INDEX payment_submission ON payments(submission_id);

CREATE TRIGGER link_reviewed_sponsorship AFTER UPDATE OF project_id,status ON submissions
WHEN NEW.status='approved' AND NEW.project_id IS NOT NULL
BEGIN
    UPDATE payments SET project_id=NEW.project_id WHERE submission_id=NEW.id AND project_id IS NULL;
END;

DROP TRIGGER fulfill_paid_feature;
CREATE TRIGGER fulfill_paid_feature AFTER UPDATE OF status,project_id ON payments
WHEN NEW.status='paid' AND NEW.project_id IS NOT NULL AND (OLD.status!='paid' OR OLD.project_id IS NULL)
BEGIN
    UPDATE projects SET featured_until=strftime('%Y-%m-%dT%H:%M:%SZ',
        max(coalesce(featured_until,''),strftime('%Y-%m-%dT%H:%M:%SZ','now')), '+7 days'),
        updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=NEW.project_id;
    INSERT INTO newsletter_features(payment_id,project_slug) SELECT NEW.id,slug FROM projects WHERE id=NEW.project_id;
END;
