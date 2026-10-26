//! Native editorial discovery; rendered and compressed through the page cache.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let guides = [
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
        "url": format!("{site_url}/guides"), "name": "Rust tool guides", "dateModified": "2026-10-26T21:38:00Z",
        "mainEntity": {"@type": "ItemList", "itemListElement": guides.iter().enumerate().map(|(i,(slug,title,_,_))| serde_json::json!({
            "@type": "ListItem", "position": i+1, "name": title, "url": format!("{site_url}/guides/{slug}")
        })).collect::<Vec<_>>()}
    });
    html! {
        article class="python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / Guides" }
            h1 { "Rust tool guides" }
            p class="guide-lead" { "Choose a Rust-powered application by the task you need to do. Our articles pair official sources with concrete checks and explicit limits." }
            @for (slug, title, date, description) in &guides {
                section {
                    h2 { a href=(format!("/guides/{slug}")) { (title) } }
                    p class="small muted" { "Published " time datetime=(date) { (date) } }
                    p { (description) }
                }
            }
            p { a href="/categories" { "Browse projects by category →" } }
            script type="application/ld+json" { (PreEscaped(schema.to_string())) }
        }
    }
}
