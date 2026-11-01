//! Native editorial discovery; rendered and compressed through the page cache.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let guides = [
        (
            "polars-csv-schema-inference",
            "Polars CSV schema: preserve IDs",
            "2026-11-01",
            "Preserve leading-zero identifiers and late mixed values with a disposable Python CSV inference exercise.",
        ),
        (
            "rustic-backup-restore",
            "Rustic backup restore: verify the files",
            "2026-10-28",
            "Restore a disposable backup and compare the recovered files instead of trusting a successful backup alone.",
        ),
        (
            "qsv-csv-validation",
            "qsv CSV validation: catch bad data before import",
            "2026-10-27",
            "Run six disposable cases that separate CSV structure, schema rules and import approval.",
        ),
        (
            "difftastic-vs-git-diff",
            "Difftastic vs git diff: keep both views",
            "2026-10-27",
            "Run four disposable comparison cases and keep a verified native patch path.",
        ),
        (
            "hurl-http-tests",
            "Hurl HTTP tests: catch a false green",
            "2026-10-25",
            "Check status, headers and JSON, then prove the test catches a broken response.",
        ),
        (
            "zed-vs-lapce",
            "Zed vs Lapce: choose by workflow",
            "2026-10-13",
            "Compare required editor tasks with a reversible migration worksheet.",
        ),
        (
            "rust-terminal-tools",
            "Rust terminal tools by task",
            "2026-10-11",
            "Choose between a terminal window, structured shell, history tool and file manager.",
        ),
        (
            "rust-python-tools",
            "Rust-powered Python tools by task",
            "2026-10-10",
            "Find where uv, Ruff and Polars fit without changing every layer at once.",
        ),
    ];
    let schema = serde_json::json!({
        "@context": "https://schema.org", "@type": "CollectionPage",
        "url": format!("{site_url}/guides"), "name": "Rust tool guides", "dateModified": "2026-11-01",
        "mainEntity": {"@type": "ItemList", "itemListElement": guides.iter().enumerate().map(|(i,(slug,title,_,_))| serde_json::json!({
            "@type": "ListItem", "position": i+1, "name": title, "url": format!("{site_url}/guides/{slug}")
        })).collect::<Vec<_>>()}
    });
    html! {
        article class="guides-index page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / Guides" }
            header class="guides-header" {
                div {
                    span class="eyebrow" { "The field guide" }
                    h1 { "Rust tool guides" }
                    p class="guide-lead" { "Find the right tool. Put it to work." }
                }
                p class="muted" { "Practical comparisons and hands-on walkthroughs for Rust-powered applications. Official sources, concrete checks and clear limits." }
            }
            div class="guides-list" {
                @for (index, (slug, title, date, description)) in guides.iter().enumerate() {
                    a class="guide-entry" href=(format!("/guides/{slug}")) {
                        div class="guide-entry-meta" {
                            span class="guide-number" { (format!("{:02}", index + 1)) }
                            time datetime=(date) { (date) }
                        }
                        div class="guide-entry-copy" {
                            h2 { (title) }
                            p { (description) }
                            span class="guide-read" { "Read guide" span aria-hidden="true" { " ↗" } }
                        }
                    }
                }
            }
            aside class="guides-directory" {
                div { h2 { "Looking for a specific tool?" } p class="muted" { "Explore the directory by category, stack or the job you need to do." } }
                a class="btn" href="/categories" { "Browse categories →" }
            }
            script type="application/ld+json" { (PreEscaped(schema.to_string())) }
        }
    }
}
