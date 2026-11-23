DROP INDEX approved_project_social;
DROP TRIGGER guard_sponsored_review;
ALTER TABLE submissions DROP COLUMN feature_requested;
DROP TRIGGER link_reviewed_sponsorship;
DROP TRIGGER fulfill_paid_feature;
DROP INDEX payment_submission;
ALTER TABLE payments DROP COLUMN submission_id;
ALTER TABLE submissions DROP COLUMN twitter_url;
ALTER TABLE submissions DROP COLUMN repo_url;
CREATE TRIGGER fulfill_paid_feature AFTER UPDATE OF status ON payments
WHEN NEW.status = 'paid' AND OLD.status != 'paid'
BEGIN
    UPDATE projects SET featured_until = strftime('%Y-%m-%dT%H:%M:%SZ',
        max(coalesce(featured_until, NEW.paid_at), NEW.paid_at), '+7 days'),
        updated_at = NEW.paid_at WHERE id = NEW.project_id;
    INSERT INTO newsletter_features(payment_id, project_slug)
        SELECT NEW.id, slug FROM projects WHERE id = NEW.project_id;
END;
