//! Database access. All SQL lives here and is checked at compile time by
//! `sqlx::query!` against the committed `.sqlx` metadata (see AGENTS.md for
//! how to regenerate it after changing a query or migration).

use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{FromRow, SqlitePool};
use std::str::FromStr;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub type Pool = SqlitePool;

/// Current UTC time as an RFC3339 string with second precision, e.g.
/// `2026-10-01T12:34:56Z`. All timestamps in the database use this format so
/// they sort and compare correctly as plain strings.
pub fn now() -> String {
    let t = OffsetDateTime::now_utc().replace_nanosecond(0).unwrap();
    t.format(&Rfc3339).unwrap()
}

pub fn now_plus_days(days: i64) -> String {
    let t = OffsetDateTime::now_utc().replace_nanosecond(0).unwrap() + time::Duration::days(days);
    t.format(&Rfc3339).unwrap()
}

pub async fn connect(database_url: &str) -> anyhow::Result<Pool> {
    let opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .pragma("cache_size", "-64000")
        .pragma("temp_store", "memory")
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new().max_connections(8).connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Category {
    pub id: i64,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CategoryCount {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Project {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub website_url: String,
    pub repo_url: Option<String>,
    pub logo_url: Option<String>,
    pub stars: Option<i64>,
    pub license: Option<String>,
    pub stack: String,
    pub category_id: Option<i64>,
    pub verified: i64,
    pub published: i64,
    pub featured_until: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub category_slug: Option<String>,
    pub category_name: Option<String>,
}

impl Project {
    pub fn stack_list(&self) -> Vec<String> {
        serde_json::from_str(&self.stack).unwrap_or_default()
    }

    pub fn is_featured(&self) -> bool {
        self.featured_until.as_deref().is_some_and(|until| until > now().as_str())
    }

    pub fn is_verified(&self) -> bool {
        self.verified != 0
    }

    pub fn domain(&self) -> String {
        self.website_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("www.")
            .split('/')
            .next()
            .unwrap_or_default()
            .to_string()
    }

    /// Logo to display: explicit `logo_url`, else the site's favicon via
    /// DuckDuckGo's icon service (no key, generous caching).
    pub fn logo(&self) -> String {
        self.logo_url.clone().unwrap_or_else(|| format!("https://icons.duckduckgo.com/ip3/{}.ico", self.domain()))
    }

    /// Outbound link with UTM so the listed site can see where traffic came from.
    pub fn outbound_url(&self) -> String {
        let sep = if self.website_url.contains('?') { '&' } else { '?' };
        format!("{}{sep}ref=builtwithrust.com", self.website_url)
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Submission {
    pub id: i64,
    pub url: String,
    pub email: Option<String>,
    pub note: Option<String>,
    pub status: String,
    pub project_id: Option<i64>,
    pub created_at: String,
    pub reviewed_at: Option<String>,
}

/// Payload accepted by the admin API and the seed file to create or update a
/// project. Only `name`, `tagline` and `website_url` are required.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectInput {
    pub name: String,
    pub tagline: String,
    pub website_url: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub stars: Option<i64>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub stack: Vec<String>,
    /// Category slug. Created on the fly if it does not exist.
    #[serde(default)]
    pub category: Option<String>,
    /// Display name used only when the category is created by this call.
    #[serde(default)]
    pub category_name: Option<String>,
    #[serde(default)]
    pub verified: bool,
    #[serde(default = "default_true")]
    pub published: bool,
}

fn default_true() -> bool {
    true
}

pub fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_dash = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

fn humanize(slug: &str) -> String {
    slug.split('-')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Categories
// ---------------------------------------------------------------------------

pub async fn categories_with_counts(pool: &Pool) -> sqlx::Result<Vec<CategoryCount>> {
    sqlx::query_as!(
        CategoryCount,
        r#"SELECT c.id AS "id!", c.slug AS "slug!", c.name AS "name!",
                  COUNT(p.id) AS "count!: i64"
           FROM categories c
           LEFT JOIN projects p ON p.category_id = c.id AND p.published = 1
           GROUP BY c.id
           HAVING COUNT(p.id) > 0
           ORDER BY c.name"#
    )
    .fetch_all(pool)
    .await
}

pub async fn category_by_slug(pool: &Pool, slug: &str) -> sqlx::Result<Option<Category>> {
    sqlx::query_as!(Category, r#"SELECT id, slug, name FROM categories WHERE slug = ?"#, slug)
        .fetch_optional(pool)
        .await
}

async fn ensure_category(pool: &Pool, slug: &str, name: Option<&str>) -> sqlx::Result<i64> {
    let slug = slugify(slug);
    if let Some(c) = category_by_slug(pool, &slug).await? {
        return Ok(c.id);
    }
    let name = name.map(str::trim).filter(|n| !n.is_empty()).map(String::from).unwrap_or_else(|| humanize(&slug));
    let rec = sqlx::query!(r#"INSERT INTO categories (slug, name) VALUES (?, ?) RETURNING id"#, slug, name)
        .fetch_one(pool)
        .await?;
    Ok(rec.id)
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

/// Build a `&'static str` SELECT over projects joined with categories, so
/// `sqlx::query_as` gets a literal (sqlx refuses runtime-built SQL strings).
macro_rules! project_select {
    ($tail:literal) => {
        concat!(
            "SELECT p.id, p.slug, p.name, p.tagline, p.description, p.website_url, p.repo_url, ",
            "p.logo_url, p.stars, p.license, p.stack, p.category_id, p.verified, p.published, ",
            "p.featured_until, p.created_at, p.updated_at, c.slug AS category_slug, c.name AS category_name ",
            "FROM projects p LEFT JOIN categories c ON c.id = p.category_id ",
            $tail
        )
    };
}

/// Published projects, optionally filtered by category and a search string.
/// Featured projects sort first, then by stars.
pub async fn list_projects(pool: &Pool, category_id: Option<i64>, q: Option<&str>) -> sqlx::Result<Vec<Project>> {
    let now = now();
    let like = q.map(|q| format!("%{}%", q.trim()));
    sqlx::query_as::<_, Project>(project_select!(
        "WHERE p.published = 1 \
           AND (?1 IS NULL OR p.category_id = ?1) \
           AND (?2 IS NULL OR p.name LIKE ?2 OR p.tagline LIKE ?2 OR p.description LIKE ?2 OR p.stack LIKE ?2) \
         ORDER BY (p.featured_until IS NOT NULL AND p.featured_until > ?3) DESC, \
                  p.stars IS NULL, p.stars DESC, p.name"
    ))
    .bind(category_id)
    .bind(like)
    .bind(now)
    .fetch_all(pool)
    .await
}

pub async fn featured_projects(pool: &Pool) -> sqlx::Result<Vec<Project>> {
    let now = now();
    sqlx::query_as::<_, Project>(project_select!(
        "WHERE p.published = 1 AND p.featured_until > ?1 ORDER BY p.featured_until DESC LIMIT 6"
    ))
    .bind(now)
    .fetch_all(pool)
    .await
}

pub async fn project_by_slug(pool: &Pool, slug: &str) -> sqlx::Result<Option<Project>> {
    sqlx::query_as::<_, Project>(project_select!("WHERE p.slug = ?1")).bind(slug).fetch_optional(pool).await
}

pub async fn related_projects(pool: &Pool, project: &Project) -> sqlx::Result<Vec<Project>> {
    sqlx::query_as::<_, Project>(project_select!(
        "WHERE p.published = 1 AND p.id != ?1 AND (?2 IS NULL OR p.category_id = ?2) \
         ORDER BY p.stars IS NULL, p.stars DESC LIMIT 6"
    ))
    .bind(project.id)
    .bind(project.category_id)
    .fetch_all(pool)
    .await
}

pub async fn all_projects(pool: &Pool) -> sqlx::Result<Vec<Project>> {
    sqlx::query_as::<_, Project>(project_select!("ORDER BY p.updated_at DESC")).fetch_all(pool).await
}

pub async fn published_slugs(pool: &Pool) -> sqlx::Result<Vec<(String, String)>> {
    let rows = sqlx::query!(r#"SELECT slug, updated_at FROM projects WHERE published = 1 ORDER BY slug"#)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| (r.slug, r.updated_at)).collect())
}

pub async fn project_count(pool: &Pool) -> sqlx::Result<i64> {
    let rec =
        sqlx::query!(r#"SELECT COUNT(*) AS "count!: i64" FROM projects WHERE published = 1"#).fetch_one(pool).await?;
    Ok(rec.count)
}

/// Insert or update a project keyed by slug. Returns the stored project.
pub async fn upsert_project(pool: &Pool, input: &ProjectInput) -> anyhow::Result<Project> {
    let slug = input.slug.as_deref().map(slugify).filter(|s| !s.is_empty()).unwrap_or_else(|| slugify(&input.name));
    anyhow::ensure!(!slug.is_empty(), "slug cannot be empty");
    anyhow::ensure!(
        input.website_url.starts_with("http://") || input.website_url.starts_with("https://"),
        "website_url must start with http:// or https://"
    );
    let category_id = match &input.category {
        Some(c) if !c.trim().is_empty() => Some(ensure_category(pool, c, input.category_name.as_deref()).await?),
        _ => None,
    };
    let stack = serde_json::to_string(&input.stack)?;
    let verified = input.verified as i64;
    let published = input.published as i64;
    let now = now();
    sqlx::query!(
        r#"INSERT INTO projects
             (slug, name, tagline, description, website_url, repo_url, logo_url, stars, license,
              stack, category_id, verified, published, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT(slug) DO UPDATE SET
             name = excluded.name, tagline = excluded.tagline, description = excluded.description,
             website_url = excluded.website_url, repo_url = excluded.repo_url,
             logo_url = excluded.logo_url, stars = excluded.stars, license = excluded.license,
             stack = excluded.stack, category_id = excluded.category_id,
             verified = excluded.verified, published = excluded.published,
             updated_at = excluded.updated_at"#,
        slug,
        input.name,
        input.tagline,
        input.description,
        input.website_url,
        input.repo_url,
        input.logo_url,
        input.stars,
        input.license,
        stack,
        category_id,
        verified,
        published,
        now,
        now,
    )
    .execute(pool)
    .await?;
    project_by_slug(pool, &slug).await?.ok_or_else(|| anyhow::anyhow!("project vanished after upsert"))
}

pub async fn delete_project(pool: &Pool, slug: &str) -> sqlx::Result<bool> {
    let res = sqlx::query!(r#"DELETE FROM projects WHERE slug = ?"#, slug).execute(pool).await?;
    Ok(res.rows_affected() > 0)
}

pub async fn extend_featured(pool: &Pool, project_id: i64, days: i64) -> sqlx::Result<()> {
    // Extend from the current expiry if still active, otherwise from now.
    let now = now();
    let until_from_now = now_plus_days(days);
    let days_str = format!("+{days} days");
    sqlx::query!(
        r#"UPDATE projects
           SET featured_until = CASE
                 WHEN featured_until IS NOT NULL AND featured_until > ?1
                   THEN strftime('%Y-%m-%dT%H:%M:%SZ', featured_until, ?2)
                 ELSE ?3 END,
               updated_at = ?1
           WHERE id = ?4"#,
        now,
        days_str,
        until_from_now,
        project_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Submissions
// ---------------------------------------------------------------------------

pub async fn create_submission(
    pool: &Pool,
    url: &str,
    email: Option<&str>,
    note: Option<&str>,
    ip_hash: Option<&str>,
) -> sqlx::Result<i64> {
    let rec = sqlx::query!(
        r#"INSERT INTO submissions (url, email, note, ip_hash) VALUES (?, ?, ?, ?) RETURNING id"#,
        url,
        email,
        note,
        ip_hash
    )
    .fetch_one(pool)
    .await?;
    Ok(rec.id)
}

/// Count of submissions from one hashed IP in the last 24h, for rate limiting.
pub async fn recent_submissions_from(pool: &Pool, ip_hash: &str) -> sqlx::Result<i64> {
    let rec = sqlx::query!(
        r#"SELECT COUNT(*) AS "count!: i64" FROM submissions
           WHERE ip_hash = ? AND created_at > strftime('%Y-%m-%dT%H:%M:%SZ','now','-1 day')"#,
        ip_hash
    )
    .fetch_one(pool)
    .await?;
    Ok(rec.count)
}

pub async fn list_submissions(pool: &Pool, status: Option<&str>) -> sqlx::Result<Vec<Submission>> {
    sqlx::query_as!(
        Submission,
        r#"SELECT id, url, email, note, status, project_id, created_at, reviewed_at
           FROM submissions WHERE (?1 IS NULL OR status = ?1)
           ORDER BY created_at DESC LIMIT 500"#,
        status
    )
    .fetch_all(pool)
    .await
}

pub async fn get_submission(pool: &Pool, id: i64) -> sqlx::Result<Option<Submission>> {
    sqlx::query_as!(
        Submission,
        r#"SELECT id, url, email, note, status, project_id, created_at, reviewed_at
           FROM submissions WHERE id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn review_submission(pool: &Pool, id: i64, status: &str, project_id: Option<i64>) -> sqlx::Result<bool> {
    let now = now();
    let res = sqlx::query!(
        r#"UPDATE submissions SET status = ?, project_id = ?, reviewed_at = ? WHERE id = ?"#,
        status,
        project_id,
        now,
        id
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Payments
// ---------------------------------------------------------------------------

pub async fn create_payment(pool: &Pool, session_id: &str, project_id: i64, email: Option<&str>) -> sqlx::Result<()> {
    sqlx::query!(
        r#"INSERT INTO payments (stripe_session_id, project_id, email) VALUES (?, ?, ?)"#,
        session_id,
        project_id,
        email
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark a payment paid. Returns the project id it was for, if the session is
/// known and was not already paid.
pub async fn mark_payment_paid(
    pool: &Pool,
    session_id: &str,
    payment_intent: Option<&str>,
    amount_cents: Option<i64>,
    currency: Option<&str>,
) -> sqlx::Result<Option<i64>> {
    let now = now();
    let rec = sqlx::query!(
        r#"UPDATE payments
           SET status = 'paid', stripe_payment_intent = ?, amount_cents = ?, currency = ?, paid_at = ?
           WHERE stripe_session_id = ? AND status != 'paid'
           RETURNING project_id"#,
        payment_intent,
        amount_cents,
        currency,
        now,
        session_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(rec.and_then(|r| r.project_id))
}

/// Record a Stripe event id. Returns `false` if it was already processed.
pub async fn record_stripe_event(pool: &Pool, id: &str, kind: &str) -> sqlx::Result<bool> {
    let res =
        sqlx::query!(r#"INSERT OR IGNORE INTO stripe_events (id, type) VALUES (?, ?)"#, id, kind).execute(pool).await?;
    Ok(res.rows_affected() > 0)
}

pub async fn payment_project_slug(pool: &Pool, session_id: &str) -> sqlx::Result<Option<String>> {
    let rec = sqlx::query!(
        r#"SELECT p.slug AS "slug?" FROM payments pay LEFT JOIN projects p ON p.id = pay.project_id
           WHERE pay.stripe_session_id = ?"#,
        session_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(rec.and_then(|r| r.slug))
}
