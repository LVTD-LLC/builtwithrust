use crate::db::Project;
use maud::{Markup, PreEscaped, html};

/// Read-only task discovery. Do not recommend unpublished catalog entries.
pub fn workflows(site_url: &str, projects: &[Project]) -> Markup {
    let choices = [
        (
            "dust",
            "Read a disk-usage tree with Dust →",
            "Inspect large files and directories in a terminal report. Check apparent-size and hard-link accounting before interpreting a total.",
        ),
        (
            "dua-cli",
            "Explore disk usage with dua →",
            "Start with an aggregate report. Its separate interactive interface can explore and delete files; this exercise never enters that mode.",
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
        "@id": format!("{site_url}/categories/productivity#disk-usage-workflows"),
        "name": "Check disk usage before choosing what to remove",
        "dateModified": "2026-11-09",
        "isPartOf": { "@type": "CollectionPage", "@id": format!("{site_url}/categories/productivity") }
    });
    html! {
        section id="disk-usage-workflows" class="task-guide" aria-labelledby="disk-usage-title" {
            h2 id="disk-usage-title" { "Check disk usage before choosing what to remove" }
            p class="guide-intro" {
                "Use Dust for a disk-usage tree or dua for aggregate reports and optional interactive exploration. "
                "Before comparing totals, match the scanned paths, size mode and hard-link policy. "
                "A sparse file’s apparent length can exceed its allocated blocks; a hard link names the same file again. "
                "Neither total alone tells you what deletion will reclaim."
            }
            nav class="workflow-guides" aria-label="Disk usage tools by task" {
                @for (slug, title, detail) in available {
                    a href={ "/projects/" (slug) } { strong { (title) } span { (detail) } }
                }
            }
            h3 { "Try the accounting difference without touching your files" }
            p class="guide-note" {
                "Built with Rust’s Linux exercise creates one small file, a hard link to it and a sparse file in a fresh temporary directory. "
                "It scans only those three explicit file paths, checks totals against Python file metadata, "
                "and verifies paths and content hashes after every scan. No user directory is accepted; no delete command is invoked. Only the generated temporary fixture is removed afterward."
            }
            p class="guide-note" {
                "In the November 9, 2026 trial with Dust 1.2.7, dua 3.0.0 and Python 3.11.2, "
                "both allocated-size reports counted the shared file once. In apparent-size mode, Dust counted both names; "
                "dua counted the shared file once unless its count-hard-links option was enabled. "
                "Compare the counting rules before treating different totals as an error. Directory overhead is excluded by using file operands."
            }
            p class="guide-note" {
                a href="/assets/examples/disk-accounting-v1/lab.py" download { "Download the read-only Python exercise →" }
                " Install trusted upstream binaries at the pinned versions, inspect the script, then run it with "
                code { "python3 lab.py --dust /absolute/path/to/dust --dua /absolute/path/to/dua" }
                ". Successful output pairs observed and expected byte totals with fixture_unchanged set to true. It downloads nothing, requires Linux sparse-file and hard-link support, and fails rather than accepting a mismatched result."
            }
            p class="guide-note" {
                "This is a small functional check, not a speed comparison, deletion-safety test or promise about reclaimable space. "
                "It does not test compression, filesystem clones, other platforms or a live cleanup. "
                "Keep exploration separate from removal; review ownership and recovery needs before deleting real data. "
                "For a terminal file manager rather than disk accounting, see the "
                a href="/guides/rust-terminal-tools" { "Rust terminal workflow guide" } "."
            }
            p class="guide-note" {
                "Flag and behavior references: "
                a href="https://github.com/bootandy/dust/tree/v1.2.7" { "Dust 1.2.7 usage" } ", "
                a href="https://github.com/Byron/dua-cli/tree/v3.0.0" { "dua 3.0.0 usage" } " and "
                a href="https://docs.python.org/3.11/library/os.html#os.stat_result" { "Python 3.11 file metadata" } ". "
                "Sources and synthetic exercise checked " time datetime="2026-11-09" { "November 9, 2026" } "."
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
