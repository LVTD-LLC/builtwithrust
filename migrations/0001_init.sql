PRAGMA journal_mode = WAL;

CREATE TABLE categories (
    id    INTEGER PRIMARY KEY,
    slug  TEXT NOT NULL UNIQUE,
    name  TEXT NOT NULL
);

CREATE TABLE projects (
    id             INTEGER PRIMARY KEY,
    slug           TEXT NOT NULL UNIQUE,
    name           TEXT NOT NULL,
    tagline        TEXT NOT NULL,
    description    TEXT NOT NULL DEFAULT '',
    website_url    TEXT NOT NULL,
    repo_url       TEXT,
    logo_url       TEXT,
    stars          INTEGER,
    license        TEXT,
    stack          TEXT NOT NULL DEFAULT '[]',   -- JSON array of crate/framework names
    category_id    INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    verified       INTEGER NOT NULL DEFAULT 0,
    published      INTEGER NOT NULL DEFAULT 1,
    featured_until TEXT,                          -- RFC3339 UTC, NULL = not featured
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_projects_category ON projects(category_id);
CREATE INDEX idx_projects_published_stars ON projects(published, stars DESC);
CREATE INDEX idx_projects_featured ON projects(featured_until);

CREATE TABLE submissions (
    id          INTEGER PRIMARY KEY,
    url         TEXT NOT NULL,
    email       TEXT,
    note        TEXT,
    status      TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','rejected')),
    project_id  INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    ip_hash     TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    reviewed_at TEXT
);
CREATE INDEX idx_submissions_status ON submissions(status, created_at);

CREATE TABLE payments (
    id                 INTEGER PRIMARY KEY,
    stripe_session_id  TEXT NOT NULL UNIQUE,
    stripe_payment_intent TEXT,
    project_id         INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    email              TEXT,
    amount_cents       INTEGER,
    currency           TEXT,
    status             TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','paid','failed')),
    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    paid_at            TEXT
);

CREATE TABLE stripe_events (
    id          TEXT PRIMARY KEY,   -- Stripe event id, for idempotency
    type        TEXT NOT NULL,
    received_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
