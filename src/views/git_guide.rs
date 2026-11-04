//! Catalog-aware Git task discovery on the existing cached Developer Tools hub.
use maud::{Markup, PreEscaped, html};

pub fn workflows(site_url: &str, projects: &[crate::db::Project]) -> Markup {
    if !projects.iter().any(|p| p.slug == "gitui") {
        return html! {};
    }
    let source = "https://github.com/gitui-org/gitui/blob/d7214eccb3cac52ce3f8d9ca18f218671821eeee/README.md";
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{site_url}/categories/developer-tools#git-workflows"),
        "name": "Review and stage Git changes in your terminal",
        "dateModified": "2026-10-24",
        "isPartOf": { "@type": "CollectionPage", "@id": format!("{site_url}/categories/developer-tools") }
    });
    html! {
        section id="git-workflows" class="task-guide" aria-labelledby="git-workflows-title" {
            h2 id="git-workflows-title" { "Review and stage Git changes in your terminal" }
            p class="guide-intro" {
                "Choose GitUI when you want a keyboard-driven view of Git changes and selective staging without leaving your terminal. "
                "It can stage files, hunks or individual lines, then commit the selected changes. Keep the Git shell available: "
                "repository features and authentication requirements can limit which tasks belong in the interface."
            }
            nav class="workflow-guides" aria-label="Git workflow tool" {
                a href="/projects/gitui" {
                    strong { "GitUI: review, stage and commit →" }
                    span { "A Rust terminal interface for Git, with commit history, branches and stashes. Open the listing for its source repository; it is not a replacement shell or a new version-control format." }
                }
            }
            details {
                summary { "Check repository fit before changing your Git workflow" }
                ul {
                    li {
                        strong { "Selective commits: " }
                        "In a disposable local repository, make two unrelated edits and stage just one hunk or line. "
                        "Inspect the staged diff in your existing Git workflow before committing; confirm the other edit stays unstaged. "
                        "Use this to check the selection behavior you need, not as proof that every repository operation is supported."
                    }
                    li {
                        strong { "Repository features: " }
                        "The checked upstream README lists no Git LFS or sparse-repository support, and places interactive rebase on the roadmap. "
                        "If these are essential, keep using a workflow that supports them rather than assuming GitUI replaces it."
                    }
                    li {
                        strong { "Authentication and policy: " }
                        "For HTTPS, upstream requires an explicitly configured credential helper. It also notes shortcomings in GPG commit signing. "
                        "Before using GitUI with a real remote, verify your authentication, required hooks and signing policy in an approved test repository. "
                        "Never paste credentials into a test file or disable a required check just to finish the trial."
                    }
                }
            }
            p class="guide-note" {
                "Read the " a href=(source) { "GitUI features, roadmap and limitations" }
                " before adopting it. This is a task-based starting point, not a performance comparison or a claim that we tested your repository. "
                "Sources checked " time datetime="2026-10-24" { "October 24, 2026" } "."
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}

/// Independent discovery path; syntax review does not depend on GitUI being listed.
pub fn syntax_review(site_url: &str, projects: &[crate::db::Project]) -> Markup {
    if !projects.iter().any(|p| p.slug == "difftastic") {
        return html! {};
    }
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{site_url}/categories/developer-tools#syntax-review-workflows"),
        "name": "Read syntax changes without losing the Git patch",
        "dateModified": "2026-11-04",
        "isPartOf": { "@type": "CollectionPage", "@id": format!("{site_url}/categories/developer-tools") }
    });
    html! {
        section id="syntax-review-workflows" class="task-guide" aria-labelledby="syntax-review-title" {
            h2 id="syntax-review-title" { "Read syntax changes without losing the Git patch" }
            p class="guide-intro" {
                "Use Difftastic as an optional syntax-aware review view, not as your staging interface or patch format. "
                "Compare it with native Git output on the same disposable change. A quiet structural diff is not proof "
                "that behavior is unchanged; keep the textual view and relevant checks."
            }
            nav class="workflow-guides" aria-label="Syntax review and patch exercise" {
                a href="/guides/difftastic-vs-git-diff" {
                    strong { "Try Difftastic vs git diff →" }
                    span {
                        "Run the four-case exercise: formatting, a changed value, significant Python indentation and parser fallback. "
                        "Check the native Git patch against the expected files."
                    }
                }
                a href="/projects/difftastic" {
                    strong { "Find Difftastic’s project and source →" }
                    span {
                        "Explore the Rust structural diff tool for reading changes. Use a separate staging workflow "
                        "to select what enters a commit."
                    }
                }
            }
            p class="guide-note" {
                "The guide’s recorded trial uses Difftastic 0.71.0 and Git 2.39.5; it is not a benchmark or a claim about every language. "
                "See "
                a href="https://github.com/Wilfred/difftastic/blob/b7d119e90ac9f972f03f69508da765dacd302c0c/manual/src/introduction.md" { "Difftastic’s syntax comparison explanation" }
                " and " a href="https://git-scm.com/docs/git-diff" { "native Git diff documentation" } ". "
                "Sources checked " time datetime="2026-11-04" { "November 4, 2026" } "."
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
