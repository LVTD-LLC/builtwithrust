//! Existing listing enrichment; every byte is rendered through the public page cache.
use maud::{Markup, PreEscaped, html};

const SOURCE: &str = "https://github.com/ekzhang/rustpad/blob/54e4a9383c84d7317af42a7ddb177ce8bcba058d/README.md";

pub fn context() -> Markup {
    let questions = [
        (
            "Does Rustpad keep documents after a restart?",
            "Not by default. Upstream documents memory-only storage by default and a one-day expiry setting, not guaranteed durable storage. Self-hosters can configure SQLITE_URI to snapshot contents to SQLite; for containers, upstream recommends keeping the database on a mounted volume. Verify recovery on your own instance rather than assuming the public service uses that configuration.",
        ),
        (
            "Is the whole Rustpad interface written in Rust?",
            "No. The server and WebAssembly text-operation logic use Rust. The surrounding browser interface uses TypeScript, React and the Monaco editor. This is a Rust-powered application, not an entirely Rust-written interface.",
        ),
    ];
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": questions.iter().map(|(question, answer)| serde_json::json!({
            "@type": "Question", "name": question,
            "acceptedAnswer": { "@type": "Answer", "text": answer }
        })).collect::<Vec<_>>()
    });
    html! {
        section class="task-guide prose" aria-labelledby="rustpad-evaluation" {
            h2 id="rustpad-evaluation" { "Is Rustpad a fit for a shared code scratchpad?" }
            p {
                "Use Rustpad to edit a short text or code example together in a browser. Treat the default server as temporary storage, not your only copy. "
                "If keeping documents matters, evaluate a self-hosted instance with SQLite persistence and verify recovery before relying on it for a continuing workflow."
            }
            p {
                "The " a href=(SOURCE) { "upstream configuration reference" } " distinguishes in-memory expiry from optional disk snapshots. "
                "Its defaults are not a statement about how the public rustpad.io service is configured. "
                "For a local project and ongoing editor work, use the "
                a href="/guides/zed-vs-lapce" { "Zed and Lapce workflow comparison" } " instead of treating a shared pad as a full development environment."
            }
            h3 { "Check these before sharing real work" }
            ol {
                li {
                    strong { "Prove the collaboration path. " }
                    "Use invented, non-confidential text. Open the same generated pad link in two separate browser profiles, edit from each, then reconnect one profile. "
                    "Check that both show the complete expected text; seeing a collaborator’s cursor alone is not enough."
                }
                li {
                    strong { "Choose a retention requirement. " }
                    "If this is a temporary discussion, keep the result in a local file before leaving. "
                    "For a continuing pad on your own instance, consult the documented " code { "SQLITE_URI" } " and " code { "EXPIRY_DAYS" }
                    " settings. Persistence of a snapshot is not a backup policy or a promise that every recent edit has reached disk."
                }
                li {
                    strong { "Test the requirement, not just the setting. " }
                    "On a disposable self-hosted instance, save a recognizable example, restart the service, reopen the exact pad link and compare its contents with your local copy. "
                    "Record editing, reconnect and recovery separately as pass, blocked or unverified. Do not restart a shared production service for this trial."
                }
            }
            p {
                "This is a suggested acceptance checklist, not a reported Rustpad deployment test, security audit or durability guarantee. "
                "Keep sensitive material out of the trial and confirm your host’s access and retention policy separately. "
                a href="/categories/productivity" { "Browse other Rust productivity tools →" }
            }
            h3 { "Storage and implementation questions" }
            @for (question, answer) in questions {
                details {
                    summary { (question) }
                    p { (answer) }
                }
            }
            p class="small muted" { "Sources checked " time datetime="2026-10-29" { "October 29, 2026" } "; checklist by Built with Rust." }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}

pub fn discovery(projects: &[crate::db::Project]) -> Markup {
    if !projects.iter().any(|project| project.slug == "rustpad") {
        return html! {};
    }
    html! {
        section id="shared-code-workflows" aria-labelledby="shared-code-workflows-title" {
            h2 id="shared-code-workflows-title" { "Edit a short code example together" }
            p { "Need a shared browser pad rather than a document viewer or typesetter? Start with temporary, non-sensitive text and decide how long you need to keep it." }
            nav class="workflow-guides" aria-label="Shared code scratchpad" {
                a href="/projects/rustpad#rustpad-evaluation" {
                    strong { "Rustpad: check collaboration and document retention →" }
                    span { "Use the two-browser checklist, distinguish default in-memory storage from optional SQLite snapshots, and keep a local copy." }
                }
            }
            p class="small muted" { "Source checked October 29, 2026. This is a separate workflow from Markdown viewing, typesetting and terminal presentations." }
        }
    }
}
