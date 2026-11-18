use crate::db::Project;
use maud::{Markup, html};

pub fn handoffs(site_url: &str, projects: &[Project]) -> Markup {
    let choices = [
        (
            "graphite",
            "Graphite: deliver an SVG",
            "Export a small icon or illustration, then open the SVG in the receiving app at its intended size and check for clipped edges. Keep the .graphite file if someone needs to edit the original node graph.",
            "https://graphite.art/learn/interface/menu-bar/",
            "Graphite file and export docs",
        ),
        (
            "rnote",
            "Rnote: return a marked-up PDF",
            "Import a two-page sample PDF with matching page sizes and mark page two. Export the whole document, then check in a separate PDF reader that both pages are in order and the note is legible.",
            "https://rnote.flxzt.net/",
            "Rnote import and export features",
        ),
        (
            "satty",
            "Satty: send an annotated screenshot",
            "Start with a sample screenshot, crop it and mark one control. Save the image, then open the saved file at the size your recipient will see. Check that the mark and enough surrounding context survived.",
            "https://github.com/Satty-org/Satty#usage",
            "Satty annotation and save docs",
        ),
    ];
    let available: Vec<_> =
        choices.into_iter().filter(|(slug, ..)| projects.iter().any(|project| project.slug == *slug)).collect();
    if available.len() < 2 {
        return html! {};
    }

    let title = "Choose a Rust design tool by what you need to share";
    let date = "2026-11-18";
    let category = format!("{site_url}/categories/design");
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{category}#design-handoffs"),
        "name": title,
        "dateModified": date,
        "isPartOf": { "@id": category, "@type": "CollectionPage" }
    });
    html! {
        section id="design-handoffs" aria-labelledby="design-handoffs-title" {
            h2 id="design-handoffs-title" { (title) }
            p class="guide-intro" {
                "Start with the file your recipient needs: a vector asset, a marked-up document or a screenshot with a callout. "
                "Try the matching tool on a throwaway file first. An export can succeed even when the recipient cannot use the result. Check the output in a separate app before relying on it."
            }
            nav class="workflow-guides" aria-label="Design tools by deliverable" {
                @for &(slug, label, check, _, _) in &available {
                    a href={ "/projects/" (slug) }
                        data-ph-capture-attribute-bwr-action="project_open"
                        data-ph-capture-attribute-bwr-project=(slug)
                        data-ph-capture-attribute-bwr-surface="design_handoffs"
                        data-ph-capture-attribute-bwr-schema="1" {
                        strong { (label) " →" }
                        span { (check) }
                    }
                }
            }
            p class="small muted source-links" {
                "Source docs checked " time datetime=(date) { "November 18, 2026" } ": "
                @for (i, &(_, _, _, url, label)) in available.iter().enumerate() {
                    @if i > 0 { ", " }
                    a href=(url) { (label) }
                }
                "."
            }
        }
        script type="application/ld+json" { (maud::PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
