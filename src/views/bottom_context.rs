use maud::{Markup, PreEscaped, html};

pub fn process_filter(site_url: &str) -> Markup {
    let url = format!("{site_url}/projects/bottom");
    let title = "Find one process when two have the same name";
    let modified = "2026-11-19";
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{url}#bottom-process-filter"),
        "name": title,
        "dateModified": modified,
        "isPartOf": { "@type": "WebPage", "@id": url }
    });

    html! {
        section id="bottom-process-filter" class="task-guide prose" aria-labelledby="bottom-process-filter-title" {
            h2 id="bottom-process-filter-title" { (title) }
            p {
                "In bottom’s process table, search by name to find candidates, then filter by process ID (PID) to check one of them. "
                "Closing search with Escape hides the input but keeps the filter. Reopen search and clear it before reading the full process list again."
            }
            ol {
                li {
                    strong { "Start with harmless duplicates. " }
                    "Open " code { "btm" } " in one Linux terminal. Run " code { "sleep 75" } " in each of two other terminals using "
                    a href="https://www.gnu.org/software/coreutils/manual/html_node/sleep-invocation.html" { "GNU sleep" }
                    ". In bottom, focus the process table. If the first column says Count rather than PID, press " kbd { "Tab" }
                    " to show individual rows. Press " kbd { "/" } ", search for " code { "sleep" } " and note two different PIDs."
                }
                li {
                    strong { "Narrow to one PID. " }
                    "With the search input open, press " kbd { "Ctrl+U" } " and replace the name with "
                    code { "pid=12345" } ", using one of the PIDs you noted. By default, the same digits can also match a longer PID. "
                    "To match the whole PID, turn on Whole Word with " kbd { "F2" } " in the search input, then verify the PID in the result."
                }
                li {
                    strong { "Restore the table. " }
                    "Press " kbd { "Esc" } ": the input closes, but the other job stays hidden. Press " kbd { "/" }
                    " again and clear the query with " kbd { "Ctrl+U" } " to restore the full list. Then search for " code { "sleep" }
                    " again to confirm both PIDs are still present. Filtering does not stop either job; if one has timed out, restart the trial."
                }
            }
            p {
                "Built with Rust checked these steps on Linux with "
                a href="https://github.com/ClementTsang/bottom/releases/tag/0.15.0" { "bottom 0.15.0" }
                " and a fresh configuration. See the "
                a href="https://bottom.pages.dev/stable/usage/widgets/process/#search" { "process search reference" }
                " for other filters, or check the "
                a href="https://github.com/ClementTsang/bottom/blob/0.15.0/src/constants.rs" { "0.15.0 key bindings" }
                " if your setup differs."
            }
            p class="small muted" { "Checked " time datetime=(modified) { "November 19, 2026" } "." }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
