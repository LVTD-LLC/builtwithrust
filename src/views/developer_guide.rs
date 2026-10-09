//! Editorial guidance rendered with the canonical category, never fetched at request time.
use maud::{Markup, PreEscaped, html};

pub const UPDATED: &str = "2026-10-09";

pub fn intro() -> Markup {
    html! {
        p class="guide-intro" {
            "Choose Rust-built developer tools by the job you need done: uv manages Python projects, "
            "Ruff checks Python code, Zed and Helix edit source, Alacritty provides a terminal, and "
            "Nushell works with structured data. These tools serve different layers of a workflow; "
            "they are not all alternatives to each other."
        }
        p class="guide-jump" { a href="#choose-by-task" { "Find a tool for your task ↓" } }
    }
}

pub fn body(site_url: &str) -> Markup {
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "CollectionPage",
        "name": "Developer Tools built with Rust",
        "url": format!("{site_url}/categories/developer-tools"),
        "hasPart": {
            "@type": "WebPageElement",
            "name": "Choose a Rust developer tool by task",
            "dateModified": UPDATED,
            "url": format!("{site_url}/categories/developer-tools#choose-by-task")
        }
    });
    html! {
        section class="task-guide" id="choose-by-task" aria-labelledby="task-guide-title" {
            h2 id="task-guide-title" { "Choose a Rust developer tool by task" }
            p class="small muted" { "Built with Rust editorial guide · Updated " time datetime=(UPDATED) { "October 9, 2026" } }
            div class="task-table-wrap" {
                table class="task-table" {
                    caption class="muted" { "Start with the task, then check the project's requirements before switching." }
                    thead { tr { th scope="col" { "Your task" } th scope="col" { "Start here" } th scope="col" { "What to know" } } }
                    tbody {
                        tr {
                            th scope="row" { "Manage Python projects" }
                            td { a href="/projects/uv" { "uv" } }
                            td { "uv manages Python dependencies, environments and Python versions. Use it when the problem is setting up or running a project. It does not take the place of a code linter: pair it with Ruff when you also want code checks. " a href="https://docs.astral.sh/uv/" { "uv documentation" } "." }
                        }
                        tr {
                            th scope="row" { "Lint and format Python" }
                            td { a href="/projects/ruff" { "Ruff" } }
                            td { "Ruff is a Python linter and formatter written in Rust. Choose it for code checks and formatting, rather than dependency management. Before replacing an existing setup, compare the rules and formatting behavior your project relies on. " a href="https://docs.astral.sh/ruff/" { "Ruff documentation" } "." }
                        }
                        tr {
                            th scope="row" { "Edit source code" }
                            td { a href="/projects/zed" { "Zed" } " or " a href="/projects/helix" { "Helix" } }
                            td { "Start with Zed for a graphical editor, or Helix for a terminal-based editor with selection-first modal editing. Choose according to where you edit and which interaction model you prefer, then check support for the languages you use. " a href="https://zed.dev/docs/" { "Zed docs" } " · " a href="https://docs.helix-editor.com/usage.html" { "Helix docs" } "." }
                        }
                        tr {
                            th scope="row" { "Run a terminal window" }
                            td { a href="/projects/alacritty" { "Alacritty" } }
                            td { "Alacritty is a terminal emulator: it displays the shell and terminal applications you run inside it. It does not replace your shell's language. Consider it when you want to change the terminal application while keeping your existing command-line workflow. " a href="https://alacritty.org/" { "Alacritty overview" } "." }
                        }
                        tr {
                            th scope="row" { "Work with structured data in a shell" }
                            td { a href="/projects/nushell" { "Nushell" } }
                            td { "Nushell’s built-in commands pass structured data through pipelines, so you can select and filter fields. Nushell has its own language; do not assume Bash scripts run unchanged. It can run inside a terminal such as Alacritty; the two solve different problems. " a href="https://www.nushell.sh/book/coming_from_bash.html" { "Nushell book" } "." }
                        }
                    }
                }
            }
            p class="guide-note" {
                "These are examples from the catalog, not a complete ranking. Rust-built tools can serve other languages: "
                "uv and Ruff work on Python projects. Open a listing for its website and source repository, "
                "then verify current platform support and licensing upstream. Directory star counts are stored snapshots, not live totals."
            }
            a href="#projects" { "Back to all developer tools ↑" }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
