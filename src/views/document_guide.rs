//! Source-checked discovery rendered inside the existing cached category.
use maud::{Markup, PreEscaped, html};

pub fn workflows(site_url: &str, projects: &[crate::db::Project]) -> Markup {
    let choices = [
        (
            "inlyne",
            "Inlyne: read and preview Markdown →",
            "Inlyne displays Markdown and a limited subset of HTML, refreshing when the file changes. Keep editing in your usual editor; check the constructs your document actually uses rather than expecting a full browser.",
        ),
        (
            "typst",
            "Typst: typeset a document →",
            "Typst is a markup-based typesetting system with math and bibliography support. Its Rust compiler and CLI can compile documents locally; the collaborative online editor is a separate way to work, not part of the open-source CLI.",
        ),
    ];
    let available: Vec<_> =
        choices.into_iter().filter(|(slug, _, _)| projects.iter().any(|p| p.slug == *slug)).collect();
    if available.is_empty() {
        return html! {};
    }
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{site_url}/categories/productivity#document-workflows"),
        "name": "Choose Rust document tools by the output you need",
        "dateModified": "2026-10-22",
        "isPartOf": { "@type": "CollectionPage", "@id": format!("{site_url}/categories/productivity") }
    });
    html! {
        section id="document-workflows" aria-labelledby="document-workflows-title" {
            h2 id="document-workflows-title" { "Choose Rust document tools by the output you need" }
            p class="guide-intro" {
                "Choose Inlyne to view an existing Markdown file while editing it elsewhere. Choose Typst to author a typeset document, such as a report with equations and references. "
                "These are different jobs, not interchangeable editors: begin with the source format and output you need, then test one representative document before switching."
            }
            p { "Keep your current workflow until every essential requirement passes; blocked or unverified requirements are reasons to continue the trial, not to switch." }
            nav class="workflow-guides" aria-label="Document tools by output" {
                @for (slug, title, detail) in available {
                    a href={ "/projects/" (slug) } {
                        strong { (title) }
                        span { (detail) }
                    }
                }
            }
            details {
                summary { "Check one document before changing your workflow" }
                ol {
                    li { "Copy a non-sensitive document and its linked files into a trial folder. Keep the original source, current viewer and expected output unchanged." }
                    li { "For Markdown viewing, check headings, tables, code blocks, relative images and links. Save an edit in your editor and confirm the preview refreshes. Check required embedded HTML explicitly; Inlyne is not a full browser and does not aim to support forms or every CSS style." }
                    li { "For typesetting, create a small Typst document with the equations, citations, fonts and page layout you need. Compile and inspect the PDF; a successful compile alone does not prove that the output meets your requirements. Do not assume existing Markdown or LaTeX source transfers unchanged." }
                    li { "Record each required feature as pass, blocked or unverified. Keep the current workflow for anything essential that fails or remains unverified; expand the trial only after the representative document works." }
                }
            }
            p class="guide-note" {
                "See " a href="https://github.com/Inlyne-Project/inlyne#features" { "Inlyne’s supported features and HTML limits" }
                ", the " a href="https://typst.app/docs/tutorial/" { "Typst document tutorial" }
                " and the " a href="https://github.com/typst/typst#readme" { "Typst compiler and CLI overview" }
                ". This is a task-based starting point, not a rendering benchmark or a completed migration test."
            }
            p class="small muted" { "Sources checked " time datetime="2026-10-22" { "October 22, 2026" } "." }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
