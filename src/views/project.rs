use super::{check_icon, external_icon, format_stars, project_grid};
use crate::db::Project;
use maud::{Markup, html};

pub fn show(p: &Project, related: &[Project]) -> Markup {
    let stack = p.stack_list();
    html! {
        article class="project" {
            div class="project-head" {
                img class="logo logo-lg" src=(p.logo()) alt="" width="56" height="56" decoding="async";
                div {
                    h1 {
                        (p.name)
                        @if p.is_verified() { span class="badge-verified" title="Verified: confirmed built with Rust" { (check_icon()) } }
                        @if p.is_featured() { span class="pill pill-featured" { "Featured" } }
                    }
                    p class="tagline-lg" { (p.tagline) }
                }
                div class="project-actions" {
                    a class="btn btn-primary" href=(p.outbound_url()) rel="noopener nofollow" target="_blank" { "Visit " (p.domain()) (external_icon()) }
                    @if let Some(repo) = &p.repo_url {
                        a class="btn" href=(repo) rel="noopener nofollow" target="_blank" { "Repository" (external_icon()) }
                    }
                }
            }
            div class="project-body" {
                div class="prose" {
                    @if p.description.is_empty() {
                        p { (p.name) " is built with Rust." }
                    } @else {
                        @for para in p.description.split("\n\n") { p { (para.trim()) } }
                    }
                }
                aside class="project-meta" {
                    dl {
                        dt { "Stars" } dd { (format_stars(p.stars)) }
                        dt { "License" } dd { (p.license.as_deref().unwrap_or("Unknown")) }
                        dt { "Category" }
                        dd {
                            @match (&p.category_slug, &p.category_name) {
                                (Some(s), Some(n)) => a href={ "/categories/" (s) } { (n) },
                                _ => "—",
                            }
                        }
                        dt { "Listed" } dd { (p.created_at.get(..10).unwrap_or(&p.created_at)) }
                    }
                    @if !stack.is_empty() {
                        h3 { "Rust stack" }
                        div class="pills" { @for s in &stack { span class="pill" { (s) } } }
                    }
                    @if !p.is_featured() {
                        a class="btn btn-small" href="/feature" { "Feature this project" }
                    }
                }
            }
            @if p.slug == "meilisearch" || p.slug == "surrealdb" {
                section class="task-guide prose database-context" aria-labelledby="source-scope-title" {
                    h2 id="source-scope-title" { "Source and deployment scope" }
                    @if p.slug == "meilisearch" {
                        p {
                            "Community and Enterprise code have different license terms. Read the "
                            a href="https://github.com/meilisearch/meilisearch/blob/541dd6e94c5d6101684610b1af1b50c5aa4e2915/LICENSE" { "repository license summary" }
                            " and the license files for your chosen version; the catalog label is not a hosted-service plan."
                        }
                    } @else {
                        p {
                            "FoundationDB support ended in 3.0.0; older deployment advice may describe 2.x. Check the "
                            a href="https://surrealdb.com/docs/reference/cli/db/commands/start" { "current storage-backend documentation" }
                            " and " a href="https://surrealdb.com/docs/manage/self-hosted/deployment-models" { "edition-specific deployment models" }
                            " before planning a deployment."
                        }
                    }
                    p {
                        a href="/categories/databases-search#database-workflows" { "Choose by task: application search or document and graph storage →" }
                    }
                    p class="muted" { "Sources checked " time datetime="2026-10-17" { "October 17, 2026" } "." }
                }
            }
            @if p.slug == "warp" {
                section class="task-guide prose warp-context" aria-labelledby="warp-tools-title" {
                    h2 id="warp-tools-title" { "Warp terminal or the warp web framework?" }
                    p {
                        "This listing is for Warp’s terminal and development environment. "
                        "If you want to build an HTTP server in Rust, the separate "
                        a href="https://docs.rs/warp/latest/warp/" { "warp web framework" }
                        " is the project to look at. Sharing a name does not make them the same tool."
                    }
                    p {
                        "For the terminal client’s licensing boundaries, read the "
                        a href="https://github.com/warpdotdev/warp#licensing" { "official repository’s licensing section" }
                        ". Do not treat the client’s open-source licenses as a promise that every hosted feature is free or open source."
                    }
                    p {
                        "Choosing a command-line setup? "
                        a href="/guides/rust-terminal-tools" { "Compare terminal, shell, history and file-manager roles →" }
                    }
                    p class="muted" { "Sources checked " time datetime="2026-10-15" { "October 15, 2026" } "." }
                }
            }
            @if p.slug == "fish" {
                section class="task-guide prose fish-context" aria-labelledby="fish-tools-title" {
                    h2 id="fish-tools-title" { "Fish, fish-lsp or rust-analyzer?" }
                    p {
                        "The shell and the language server are different projects. Fish 4.0 moved the shell’s C++ implementation to Rust; "
                        "that does not make its scripts, supporting tools or editor integrations Rust programs. "
                        a href="https://fishshell.com/blog/rustport/" { "Read the Fish team’s account of the port." }
                    }
                    ul {
                        li {
                            strong { "Want an interactive shell? " }
                            "Use Fish for running commands, with built-in autosuggestions and tab completion. "
                            "After installing it, try a session by running " code { "fish" } "; use " code { "exit" }
                            " to return to your previous shell. You do not need to change your login shell for this trial. "
                            a href="https://fishshell.com/docs/current/index.html#starting-and-exiting" { "Fish setup instructions" }
                        }
                        li {
                            strong { "Editing .fish scripts? " }
                            a href="https://www.fish-lsp.dev/docs/introduction" { "fish-lsp" }
                            " adds editor features for the Fish language. It is a separate "
                            a href="https://www.fish-lsp.dev/docs/how-it-works" { "TypeScript language server" }
                            ", not the Rust rewrite of the shell. Follow its client-specific setup; Fish’s interactive completions are a different feature."
                        }
                        li {
                            strong { "Editing Rust source files? " }
                            a href="https://rust-analyzer.github.io/" { "rust-analyzer" }
                            " provides language-server support for Rust code. Choose the server for the language you are editing, not the language your shell was built with."
                        }
                    }
                    p {
                        "Choosing the rest of your command-line setup? "
                        a href="/guides/rust-terminal-tools" { "Compare terminal, shell, history and file-manager roles →" }
                    }
                    p class="muted" { "Sources checked " time datetime="2026-10-14" { "October 14, 2026" } "." }
                }
            }
            @if matches!(p.slug.as_str(), "uv" | "ruff" | "polars") {
                section class="task-guide" aria-labelledby="python-guide-title" {
                    h2 id="python-guide-title" { "Where this fits in a Python workflow" }
                    a href="/guides/rust-python-tools" { "Read the guide to Rust-powered Python tools: uv, Ruff and Polars →" }
                }
            }
            @if matches!(p.slug.as_str(), "alacritty" | "nushell" | "atuin" | "yazi") {
                section class="task-guide" aria-labelledby="terminal-guide-title" {
                    h2 id="terminal-guide-title" { "Where this fits in a terminal workflow" }
                    a href="/guides/rust-terminal-tools" { "Read the guide to Rust terminal tools: Alacritty, Nushell, Atuin and Yazi →" }
                }
            }
            @if matches!(p.slug.as_str(), "zed" | "lapce") {
                section class="task-guide" aria-labelledby="editor-guide-title" {
                    h2 id="editor-guide-title" { "Is this editor a fit for your workflow?" }
                    a href="/guides/zed-vs-lapce" { "Compare Zed and Lapce by language support, remote work and collaboration →" }
                }
            }
            @if !related.is_empty() {
                section class="related" {
                    h2 { "More like this" }
                    (project_grid(related))
                }
            }
        }
    }
}
