-- One entitlement per Checkout Session, even across duplicate Stripe events.
CREATE TABLE newsletter_features (
    payment_id INTEGER PRIMARY KEY REFERENCES payments(id),
    project_slug TEXT NOT NULL,
    fulfilled_issue TEXT
);

CREATE TRIGGER fulfill_paid_feature AFTER UPDATE OF status ON payments
WHEN NEW.status = 'paid' AND OLD.status != 'paid'
BEGIN
    UPDATE projects SET featured_until = strftime('%Y-%m-%dT%H:%M:%SZ',
        max(coalesce(featured_until, NEW.paid_at), NEW.paid_at), '+7 days'),
        updated_at = NEW.paid_at WHERE id = NEW.project_id;
    INSERT INTO newsletter_features(payment_id, project_slug)
        SELECT NEW.id, slug FROM projects WHERE id = NEW.project_id;
END;
