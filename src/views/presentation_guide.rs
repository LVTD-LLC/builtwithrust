//! Task-led discovery on the existing cached Productivity category.
use maud::{Markup, PreEscaped, html};

pub fn workflows(site_url: &str, projects: &[crate::db::Project]) -> Markup {
    if !projects.iter().any(|p| p.slug == "presenterm") {
        return html! {};
    }
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{site_url}/categories/productivity#presentation-workflows"),
        "name": "Present a Markdown slide deck in your terminal",
        "dateModified": "2026-10-23",
        "isPartOf": { "@type": "CollectionPage", "@id": format!("{site_url}/categories/productivity") }
    });
    html! {
        section id="presentation-workflows" class="task-guide" aria-labelledby="presentation-workflows-title" {
            h2 id="presentation-workflows-title" { "Present a Markdown slide deck in your terminal" }
            p class="guide-intro" {
                "For a talk built around Markdown and code, presenterm turns a slide deck into a terminal presentation. "
                "It is a slide presenter, not a general Markdown document viewer. Rehearse the actual display setup and check the shareable export before committing to it for a talk."
            }
            nav class="workflow-guides" aria-label="Terminal presentation tool" {
                a href="/projects/presenterm" {
                    strong { "presenterm: present Markdown slides →" }
                    span { "A Rust slideshow tool with themes, syntax-highlighted code and terminal-dependent image support. Start with one representative deck, not a full conversion of your existing slides." }
                }
            }
            details {
                summary { "Rehearse the display, handoff and code settings" }
                ol {
                    li {
                        strong { "Display: " }
                        "Use a copy of a non-sensitive deck with representative code and local images. Check text size, line wrapping and image paths in the terminal and display setup you will actually present with, including tmux if used. "
                        "Image rendering depends on terminal protocols; an ASCII fallback is not the same result as full image support. "
                        a href="https://mfontanini.github.io/presenterm/features/images.html" { "Check image support and tmux requirements." }
                    }
                    li {
                        strong { "Handoff: " }
                        "Export and open the file you plan to share. HTML export produces a self-contained file; PDF export requires WeasyPrint and its dependencies. "
                        "Check every required slide in the exported file rather than assuming it matches the terminal presentation. "
                        a href="https://mfontanini.github.io/presenterm/features/exports.html" { "Read the export requirements." }
                    }
                    li {
                        strong { "Code settings: " }
                        "Keep snippet execution and automatic execution disabled unless you intend to run reviewed code. Check both command-line options and existing configuration. "
                        "Snippet validation can also run code and rerun executable snippets on reload; highlighting code is a different task from executing it. "
                        a href="https://mfontanini.github.io/presenterm/features/code/execution.html" { "Review execution and validation behavior." }
                    }
                    li {
                        strong { "Decision: " }
                        "Mark display, export and code settings as pass, blocked or unverified. Keep your current presentation and a checked fallback until every essential requirement passes. "
                        "This is a rehearsal checklist, not a claim that we tested your terminal, projector or deck."
                    }
                }
            }
            p class="guide-note" {
                "See the " a href="https://github.com/mfontanini/presenterm#readme" { "presenterm feature overview" }
                " for supported features. Sources checked " time datetime="2026-10-23" { "October 23, 2026" } "."
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
