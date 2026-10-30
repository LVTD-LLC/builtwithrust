//! Task entry into the existing CSV exercise, not a second tutorial.
use crate::db::Project;
use maud::{Markup, html};

pub fn workflows(projects: &[Project]) -> Markup {
    if !projects.iter().any(|p| p.slug == "qsv") {
        return html! {};
    }
    html! {
        section id="csv-import-workflows" aria-labelledby="csv-import-title" {
            h2 id="csv-import-title" { "Check a CSV before import" }
            p class="guide-intro" {
                "Separate three questions before importing a CSV: can the file be parsed, do its rows meet your reviewed schema, "
                "and will the destination accept the data? Start with a disposable copy. A successful parser or schema check "
                "does not settle every destination rule, such as uniqueness."
            }
            nav class="workflow-guides" aria-label="CSV import checks" {
                a href="/guides/qsv-csv-validation" {
                    strong { "Try the CSV acceptance exercise →" }
                    span { "Use six synthetic qsv 26.0.0 cases to compare parser failures, schema failures and a duplicate-ID case that passes the example schema." }
                }
                a href="/projects/qsv" {
                    strong { "Explore the command-line tool →" }
                    span { "Find qsv’s project and source before deciding whether a separate shell-based check fits your workflow." }
                }
            }
            p class="guide-note" {
                "Keep the original file unchanged during a trial. Review rejected rows before choosing an all-or-nothing or partial import policy; "
                "a valid subset is not automatic permission to load it. Then check the destination’s key, relationship and identifier-preservation requirements separately. "
                "The exercise tests a small synthetic contract, not production compatibility."
            }
            p class="small muted" {
                "Based on our " a href="/guides/qsv-csv-validation#exercise" { "published exercise and its limits" }
                " and the " a href="https://github.com/dathere/qsv/blob/26.0.0/docs/help/validate.md" { "qsv 26.0.0 validation reference" }
                ". Checked " time datetime="2026-10-30" { "October 30, 2026" } "."
            }
        }
    }
}

pub fn polars_context() -> Markup {
    html! {
        section class="task-guide" aria-labelledby="polars-csv-checks" {
            h2 id="polars-csv-checks" { "Bringing a CSV into your workflow?" }
            p {
                "If you want a separate command-line acceptance check before loading data, "
                a href="/guides/qsv-csv-validation" { "try the qsv CSV validation exercise" }
                ". Its example separates readable CSV from schema-valid rows, and shows why duplicate IDs still need a destination-specific rule. "
                "This is an optional preflight, not a requirement to use Polars or a replacement for checks in your own program."
            }
        }
    }
}
