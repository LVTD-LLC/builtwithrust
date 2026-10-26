//! Editorial guidance rendered with the canonical category, never fetched at request time.
use maud::{Markup, PreEscaped, html};

pub const UPDATED: &str = "2026-10-12";

pub fn javascript_workflows(site_url: &str, projects: &[crate::db::Project]) -> Markup {
    let choices = [
        (
            "biome",
            "Biome: lint and format JavaScript →",
            "Biome checks and formats source. Trial it without changing your bundler; compare required lint rules and formatting diffs before retiring an existing configuration.",
        ),
        (
            "rspack",
            "Rspack: trial a webpack-style build →",
            "Rspack bundles applications with webpack-oriented configuration. Check the loaders and plugins your project uses; its SWC loader transforms TypeScript but does not type-check it.",
        ),
        (
            "turbopack",
            "Turbopack: check Next.js bundling →",
            "Turbopack is a Rust-based bundler integrated into Next.js. Evaluate it against your installed Next.js version and custom build configuration, not as a generic linter or a drop-in webpack configuration swap.",
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
        "@id": format!("{site_url}/categories/developer-tools#javascript-workflows"),
        "name": "Choose Rust-powered JavaScript tools by task",
        "dateModified": "2026-10-21",
        "isPartOf": { "@type": "CollectionPage", "@id": format!("{site_url}/categories/developer-tools") }
    });
    html! {
        section id="javascript-workflows" aria-labelledby="javascript-workflows-title" {
            h2 id="javascript-workflows-title" { "Choose Rust-powered JavaScript tools by task" }
            p class="guide-intro" {
                "Start with the layer you want to change: linting and formatting, an application build, or Next.js bundling. "
                "A source-checking tool and a bundler can work together. Adopting one does not require replacing the rest of your toolchain."
            }
            nav class="workflow-guides" aria-label="JavaScript tools by workflow" {
                @for (slug, title, detail) in available {
                    a href={ "/projects/" (slug) } {
                        strong { (title) }
                        span { (detail) }
                    }
                }
            }
            details {
                summary { "Try one change without losing a check" }
                ol {
                    li { "Create a trial branch and record the current tool versions, lint command, type-check command and production build command. Keep the original configuration available." }
                    li { "For linting and formatting, compare the reports and proposed diffs on representative files. Check essential rules and ignored files; automatic migration does not guarantee identical coverage." }
                    li { "For bundling, exercise one real application route and a source edit, then build and run the production output. Check assets, imports and required loaders or plugins in both modes, not just whether the development server starts." }
                    li { "Keep TypeScript type checking explicit. Rspack’s SWC loader and Turbopack do not perform it themselves; verify that your framework or CI still rejects a deliberate type error on the trial branch, then undo the error." }
                    li { "Keep the change only if required checks and the built application still behave as intended. Otherwise return to the original configuration; evaluate speed separately on your own workload." }
                }
            }
            p class="guide-note" {
                "Read the official " a href="https://biomejs.dev/introduction/comparison-with-other-tools/" { "Biome compatibility notes" }
                ", " a href="https://www.rspack.dev/guide/migration/webpack" { "Rspack migration guide" }
                ", " a href="https://www.rspack.dev/guide/languages/typescript" { "Rspack TypeScript guidance" }
                " and " a href="https://nextjs.org/docs/app/api-reference/turbopack" { "Turbopack reference" }
                " for your versions. These are task-based starting points, not a performance ranking or a hands-on migration report."
            }
            p class="small muted" { "Sources checked " time datetime="2026-10-21" { "October 21, 2026" } "." }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}

pub fn intro() -> Markup {
    html! {
        p class="guide-intro" {
            "Choose Rust-built developer tools by the job you need done: uv manages Python projects, "
            "Ruff checks Python code, Zed and Helix edit source, Alacritty provides a terminal, and "
            "Nushell works with structured data. These tools serve different layers of a workflow; "
            "they are not all alternatives to each other."
        }
        p class="guide-jump" { a href="#choose-by-task" { "Find a tool for your task ↓" } }
        nav class="workflow-guides" aria-label="Developer workflow guides" {
            a href="/guides/rust-python-tools" {
                strong { "Choose a Python workflow tool →" }
                span { "uv, Ruff or Polars? Compare project management, code checks and DataFrame work." }
            }
            a href="/guides/rust-terminal-tools" {
                strong { "Choose a terminal workflow tool →" }
                span { "Alacritty, Nushell, Atuin or Yazi? Find one useful change, with a trial and an undo path." }
            }
        }
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
            p class="small muted" { "Built with Rust editorial guide · Updated " time datetime=(UPDATED) { "October 12, 2026" } }
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

/// A discovery path into the existing exercise, not another HTTP tutorial.
pub fn http_workflows(site_url: &str, projects: &[crate::db::Project]) -> Markup {
    if !projects.iter().any(|p| p.slug == "hurl") {
        return html! {};
    }
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{site_url}/categories/developer-tools#http-workflows"),
        "name": "Test HTTP responses, not just status codes",
        "dateModified": "2026-10-26",
        "isPartOf": { "@type": "CollectionPage", "@id": format!("{site_url}/categories/developer-tools") }
    });
    html! {
        section id="http-workflows" class="task-guide" aria-labelledby="http-workflows-title" {
            h2 id="http-workflows-title" { "Test HTTP responses, not just status codes" }
            p class="guide-intro" {
                "Start with Hurl when you want HTTP requests and repeatable response assertions in plain-text files. "
                "An HTTP 200 check can still pass when the JSON body is wrong. Choose the headers and fields your caller needs, "
                "then deliberately break one to check that your test rejects the response."
            }
            nav class="workflow-guides" aria-label="HTTP testing starting points" {
                a href="/guides/hurl-http-tests" {
                    strong { "Try the local Hurl HTTP testing exercise →" }
                    span { "Download a loopback fixture and compare a status-only check with header and JSON assertions. Follow the failure matrix to distinguish a wrong response from a server you cannot reach." }
                }
                a href="/projects/hurl" {
                    strong { "Check Hurl's project and source links →" }
                    span { "Open the catalog listing for the official website and source repository. Hurl is a Rust CLI powered by libcurl, not an entirely Rust networking stack." }
                }
            }
            p class="guide-note" {
                "Read Hurl's " a href="https://hurl.dev/docs/asserting-response.html" { "response-assertion reference" }
                " for the checks it supports. This exercise stays at the HTTP layer: it does not run browser JavaScript "
                "or certify a service's load capacity or security. Sources checked "
                time datetime="2026-10-26" { "October 26, 2026" } "."
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
