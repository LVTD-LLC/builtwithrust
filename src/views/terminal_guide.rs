//! Static editorial content: rendered and compressed once through the page cache.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/rust-terminal-tools");
    let faqs = [
        (
            "Is Alacritty an alternative to Nushell?",
            "Not directly. Alacritty is a terminal emulator; Nushell is a shell and language. You can run Nushell inside Alacritty, or keep your existing shell when trying Alacritty.",
        ),
        (
            "Does Atuin require cloud sync?",
            "No. Atuin supports local history storage and search without enabling sync. Sync is an optional, separate decision; local history can still contain sensitive commands and context.",
        ),
        (
            "Why does my shell stay in the same directory after I quit Yazi?",
            "Navigating inside Yazi does not by itself change its parent shell’s working directory. Use the shell-specific wrapper from Yazi’s official quick-start guide if you want that behavior.",
        ),
    ];
    let schema = serde_json::json!({
        "@context": "https://schema.org", "@graph": [
            {"@type": "Article", "@id": url, "url": url, "headline": "Rust terminal tools: choose your next step",
             "datePublished": "2026-10-11", "dateModified": "2026-10-11",
             "author": {"@type": "Organization", "name": "Built with Rust", "url": site_url}},
            {"@type": "BreadcrumbList", "itemListElement": [
                {"@type": "ListItem", "position": 1, "name": "Projects", "item": site_url},
                {"@type": "ListItem", "position": 2, "name": "Rust terminal tools", "item": url}
            ]},
            {"@type": "FAQPage", "mainEntity": faqs.iter().map(|(q,a)| serde_json::json!({
                "@type": "Question", "name": q, "acceptedAnswer": {"@type": "Answer", "text": a}
            })).collect::<Vec<_>>()}
        ]
    });
    html! {
        article class="terminal-guide python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / Rust terminal tools" }
            h1 { "Rust terminal tools: choose your next step" }
            p class="guide-lead" {
                "Choose Rust terminal tools by the problem you want to solve: Alacritty changes the terminal window, Nushell changes how you work with structured data, Atuin helps retrieve command history, and Yazi helps navigate files. Try one change while keeping the rest of your setup. These projects are complementary, not interchangeable terminals."
            }
            p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-10-11" { "October 11, 2026" } }
            p {
                "An all-Rust setup is not the goal here. A useful change solves a problem you can name without breaking something you rely on. "
                "This guide compares four catalog projects through a small trial, an acceptance check and a way back. It is an editorial selection framework, "
                "not a benchmark or a report that we tested every combination. Keep the tools that already work for you."
            }
            nav aria-label="In this guide" class="small" {
                a href="#choose" { "Choose a task" } " · " a href="#alacritty" { "Alacritty" } " · "
                a href="#nushell" { "Nushell" } " · " a href="#atuin" { "Atuin" } " · "
                a href="#yazi" { "Yazi" } " · " a href="#trial" { "Trial worksheet" }
            }
            section aria-labelledby="choose" {
                h2 id="choose" { "Choose Rust terminal tools by the smallest useful change" }
                p {
                    "Separate the window from the work inside it. A terminal emulator displays a shell and the applications you launch. "
                    "The shell interprets commands. History integration helps recover past commands; a file manager helps browse directories. "
                    "Changing one does not automatically improve the others. Start with the symptom in the first column, not a list of packages to install."
                }
                figure class="python-layers" {
                    svg viewBox="0 0 380 330" width="100%" role="img" aria-labelledby="terminal-trial-title terminal-trial-desc" {
                        title id="terminal-trial-title" { "A reversible terminal-tool trial" }
                        desc id="terminal-trial-desc" { "Name one problem, change one layer, repeat the task, then keep the change or return to the working setup." }
                        @for (y, title, detail) in [(4, "1. Name the friction", "Window, data, history or files?"), (94, "2. Change one layer", "Keep the rest of the setup"), (184, "3. Repeat the task", "Keep the improvement or undo")] {
                            rect x="1" y=(y) width="378" height="72" rx="10" fill="var(--bg-soft)" stroke="var(--border-strong)" {}
                            text x="18" y=(y + 28) fill="var(--accent)" font-size="21" font-weight="700" { (title) }
                            text x="18" y=(y + 54) fill="currentColor" font-size="17" { (detail) }
                        }
                        path d="M190 77 v15 m-5 -6 l5 6 l5 -6 M190 167 v15 m-5 -6 l5 6 l5 -6" fill="none" stroke="currentColor" stroke-width="2" {}
                        text x="18" y="284" fill="currentColor" font-size="15" { "Keep your original setup available." }
                        text x="18" y="314" fill="var(--fg-muted)" font-size="12" { "Built with Rust · October 2026" }
                    }
                    figcaption class="small muted" { "A successful trial can end with keeping your existing tools." }
                }
                p class="small muted python-table-hint" { "Scroll sideways to read the trial and acceptance check." }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Terminal tool selection table, scroll horizontally on small screens" {
                    table class="task-table" {
                        caption { "One problem, one trial, one decision" }
                        thead { tr { th scope="col" { "Your friction" } th scope="col" { "Try" } th scope="col" { "Keep unchanged" } th scope="col" { "Acceptance check" } } }
                        tbody {
                            tr { th scope="row" { "The terminal window does not suit you" } td { a href="/projects/alacritty" { "Alacritty" } } td { "Shell, scripts and editor" } td { "Your usual text, keys and selection work comfortably" } }
                            tr { th scope="row" { "Extracting fields means parsing text" } td { a href="/projects/nushell" { "Nushell" } } td { "Default shell and existing scripts" } td { "One sample produces the expected fields and values" } }
                            tr { th scope="row" { "You cannot find a past command" } td { a href="/projects/atuin" { "Atuin" } } td { "Terminal and shell language" } td { "You find a harmless test command without rerunning it" } }
                            tr { th scope="row" { "Browsing files is awkward" } td { a href="/projects/yazi" { "Yazi" } } td { "Shell, editor and real project files" } td { "You locate a sample file and understand where your shell returns" } }
                        }
                    }
                }
            }
            section aria-labelledby="alacritty" {
                h2 id="alacritty" { "Alacritty: change the terminal, not your command language" }
                p {
                    a href="/projects/alacritty" { "Alacritty" } " is a terminal emulator. Its "
                    a href="https://alacritty.org/" { "official overview" } " describes scrollback search, vi-mode navigation and multiple windows. "
                    "It is a sensible candidate when the terminal application's interaction is what you want to change, while retaining the shell and tools you already use. "
                    "Installing it is not a way to make Bash understand Nushell syntax."
                }
                p {
                    "For a first trial, open your existing shell in a separate Alacritty window. Repeat a familiar, harmless task: "
                    "read a log sample, select and copy text, search scrollback, and open your usual terminal editor. Check the actual keyboard layout, "
                    "font and display you use. The decision is whether the interface suits that work, not whether a screenshot looks cleaner. "
                    "Leave your current terminal as the default until those checks are satisfactory."
                }
                p {
                    "The tradeoff is deliberately limited built-in scope. The " a href="https://github.com/alacritty/alacritty" { "Alacritty project README" }
                    " excludes built-in tabs and splits. If those are essential, factor in a separate multiplexer or keep a terminal that already provides them. "
                    "That is a legitimate reason not to switch. To end this trial, close the new window and return to the old application; "
                    "there is no need to migrate shell scripts to evaluate the display layer."
                }
            }
            section aria-labelledby="nushell" {
                h2 id="nushell" { "Nushell: try structured data without migrating your shell" }
                p {
                    a href="/projects/nushell" { "Nushell" } " is a shell and language built around structured data. "
                    "Its built-in commands can pass records and tables rather than requiring every step to reconstruct fields from formatted text. "
                    "The " a href="https://www.nushell.sh/book/coming_from_bash.html" { "guide for Bash users" } " shows how familiar operations map to Nu. "
                    "This is a data-manipulation choice, not a replacement for the window around your shell."
                }
                p {
                    "Use a temporary Nu session and a disposable JSON or CSV sample with a few known values. Select the fields you need, "
                    "filter one known case, and compare the result with the expected records. A successful trial means you can explain both the "
                    "transformation and its output. Keep your default shell, deployment scripts and CI jobs unchanged while you learn the model. "
                    "Exit the trial session if the benefit does not justify learning another language."
                }
                p {
                    "The compatibility boundary matters: " a href="https://www.nushell.sh/book/thinking_in_nu.html" { "Nushell is not a drop-in Bash replacement" }
                    ". Do not paste Bash initialization or production scripts into Nu and assume they behave identically. Keep those scripts under "
                    "their intended interpreter. Also, external programs do not automatically emit Nu tables: the "
                    a href="https://www.nushell.sh/book/pipelines.html" { "pipeline documentation" } " distinguishes structured values from external byte streams. "
                    "The strongest case for trying Nu is repeated work with structured fields; a working Bash automation estate alone is not a migration problem."
                }
            }
            section aria-labelledby="atuin" {
                h2 id="atuin" { "Atuin: fix command recall before changing terminals" }
                p {
                    "If you like your terminal but keep losing a command you used yesterday, start with history. " a href="/projects/atuin" { "Atuin" }
                    " stores and searches shell history with context. Its " a href="https://docs.atuin.sh/" { "documentation" }
                    " explicitly supports local use without sync. Its " a href="https://docs.atuin.sh/main/guide/shell-integration/" { "shell integration" }
                    " uses hooks to record commands and context such as the working directory. A different terminal window is not a substitute for loading those hooks correctly."
                }
                p {
                    "Check the integration instructions for your actual shell, preserve its current configuration, and use a fresh session for the trial. "
                    "Record a distinctive, harmless test command, then search for it. Inspect the result without executing it: the docs distinguish "
                    "selecting a command for editing from running it. The acceptance check is successful recall in your intended shell, "
                    "not importing your entire old history or creating a sync account."
                }
                p {
                    "History is sensitive even when it stays local. It can contain commands and working-directory context you do not want to retain or share. "
                    "Review the project's storage and privacy controls before importing old history. Treat "
                    a href="https://docs.atuin.sh/latest/guide/sync/" { "optional cross-device sync" } " as a separate decision after local search is useful. "
                    "If you stop the trial, remove the trial initialization following the " a href="https://docs.atuin.sh/latest/uninstall/" { "official uninstall guidance" }
                    " then close the trial sessions and reopen the shell. Removing integration is not the same as deleting stored history; decide separately what data to retain."
                }
            }
            section aria-labelledby="yazi" {
                h2 id="yazi" { "Yazi: separate file browsing from shell navigation" }
                p {
                    a href="/projects/yazi" { "Yazi" } " is a terminal file manager. It is useful to evaluate when moving through folders and locating files "
                    "is the task you want to improve, rather than changing your command language or command history. Start in a disposable sample folder. "
                    "Browse into a subdirectory, find a known file and quit without moving or deleting anything. Follow the "
                    a href="https://yazi-rs.github.io/docs/quick-start/" { "official quick start" } " for navigation and your shell's integration."
                }
                p {
                    "Check two outcomes separately: did the file browser help you locate the file, and did your shell end up where you expected? "
                    "Plain navigation inside Yazi does not change the parent shell's directory. Its shell-specific wrapper supplies that behavior on exit. "
                    "You can evaluate browsing first, then add the appropriate wrapper only if returning to the selected directory is useful. "
                    "Do not copy one shell's wrapper into a different shell's configuration."
                }
                p {
                    "Previews introduce another boundary. The " a href="https://yazi-rs.github.io/docs/installation/" { "installation documentation" }
                    " lists optional tools used for preview and search features, while the " a href="https://yazi-rs.github.io/docs/image-preview/" { "image-preview documentation" }
                    " explains terminal support. In particular, Alacritty does not provide native image display; supported auxiliary arrangements depend on the environment. "
                    "Do not assume two Rust tools form a complete image-preview setup. Keep your current file workflow if a required preview does not work. "
                    "To undo a browsing-only trial, quit Yazi; restore any wrapper changes separately if you made them."
                }
            }
            section aria-labelledby="trial" {
                h2 id="trial" { "A keep-or-undo worksheet for Rust terminal tools" }
                p {
                    "Before changing a default or importing data, write down the following four lines. They make the trial specific enough to reject "
                    "a tool that is interesting but does not solve your problem. Use the same task before and after, with non-sensitive sample data."
                }
                ol {
                    li { strong { "The problem: " } "Name one repeated interruption, such as failing to recover a command, not a vague wish for a modern terminal." }
                    li { strong { "The boundary: " } "Name the only component you will change and what stays untouched. Keep the original application and configuration available." }
                    li { strong { "The acceptance check: " } "Describe an observable result: the right record survives a filter, a command is found, or the expected file is reached. Include the compatibility feature you cannot lose." }
                    li { strong { "The exit: " } "Choose how to return to the previous setup before expanding the trial. Separate configuration rollback from decisions about any history or other data the new tool stored." }
                }
                p {
                    "For example, if command recall is the only problem, your first candidate is Atuin, not a new terminal plus a new shell. "
                    "Leave the terminal and shell language unchanged, check recall with a harmless command, then decide whether local history search is worth keeping. "
                    "Only after that decision should you evaluate sync. If your real requirement is image previews, start instead with Yazi's compatibility documentation "
                    "and the terminal you actually use. The two problems lead to different trials, even though both happen at a command line."
                }
            }
            section aria-labelledby="questions" {
                h2 id="questions" { "Common selection questions" }
                @for (q,a) in faqs { h3 { (q) } p { (a) } }
            }
            section aria-labelledby="next-step" {
                h2 id="next-step" { "Open the project that fits your task" }
                p {
                    "Explore " a href="/projects/alacritty" { "Alacritty's terminal listing" } ", " a href="/projects/nushell" { "Nushell's shell listing" }
                    ", " a href="/projects/atuin" { "Atuin's history tooling" } " or " a href="/projects/yazi" { "Yazi's file manager" }
                    " for upstream websites and source repositories. For other jobs, browse the " a href="/categories/developer-tools" { "developer-tools catalog" }
                    ". A useful project is missing? " a href="/submit" { "Submit it for review" } "."
                }
                p class="small muted" {
                    "Scope: four projects in the Built with Rust catalog, checked against official documentation on October 11, 2026. "
                    "The trial worksheet is Built with Rust's editorial recommendation, not a tested performance ranking. "
                    "Check current platform and installation requirements upstream; directory star counts are stored snapshots."
                }
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
