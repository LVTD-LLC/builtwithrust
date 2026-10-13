//! Documentation-backed comparison; rendered once through the shared page cache.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/zed-vs-lapce");
    let faqs = [
        (
            "Is Zed or Lapce better for Rust?",
            "Zed documents native Rust support through rust-analyzer; Lapce offers a rust-analyzer plugin. Choose by the workspace operations you need, including navigation, diagnostics, formatting and debugging. This guide does not establish a performance winner.",
        ),
        (
            "Can I use Zed and Lapce on Windows?",
            "Both projects provide Windows downloads, alongside macOS and Linux. Check each editor's current installation requirements and your graphics hardware; the operating-system name alone does not establish compatibility.",
        ),
        (
            "Is Lapdev required to use Lapce?",
            "No. Lapce is the code editor; Lapdev is a separate development-environment product. Do not treat Lapdev's hosted pricing as the price of installing Lapce.",
        ),
    ];
    let schema = serde_json::json!({
        "@context": "https://schema.org", "@graph": [
            {"@type": "Article", "@id": url, "url": url, "headline": "Zed vs Lapce: choose by workflow",
             "datePublished": "2026-10-13", "dateModified": "2026-10-13",
             "author": {"@type": "Organization", "name": "Built with Rust", "url": site_url}},
            {"@type": "BreadcrumbList", "itemListElement": [
                {"@type": "ListItem", "position": 1, "name": "Projects", "item": site_url},
                {"@type": "ListItem", "position": 2, "name": "Zed vs Lapce", "item": url}
            ]},
            {"@type": "FAQPage", "mainEntity": faqs.iter().map(|(q,a)| serde_json::json!({
                "@type": "Question", "name": q, "acceptedAnswer": {"@type": "Answer", "text": a}
            })).collect::<Vec<_>>()}
        ]
    });
    html! {
        article class="editor-guide python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / Zed vs Lapce" }
            h1 { "Zed vs Lapce: choose by workflow" }
            p class="guide-lead" {
                "Try Zed first for documented native Rust and Python language workflows. Try Lapce if its toggleable modal editing and Apache-licensed source fit your priorities. Both offer desktop editing and remote development. Before switching, test your essential language, extension and SSH tasks in one real workspace; neither editor wins merely by being written in Rust."
            }
            p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-10-13" { "October 13, 2026" } }
            p {
                "A missing debugger workflow can matter more than a dozen attractive features. Start with what would stop you working, then compare preferences. "
                "This is a documentation-based decision guide with a reusable acceptance worksheet, not a hands-on benchmark. "
                "Our catalog entries for " a href="/projects/zed" { "Zed" } " and " a href="/projects/lapce" { "Lapce" }
                " link to the projects; the comparison below helps you decide which to investigate first."
            }
            nav aria-label="In this guide" class="small" {
                a href="#compare" { "Quick comparison" } " · " a href="#languages" { "Rust and Python" } " · "
                a href="#remote" { "SSH work" } " · " a href="#extensions" { "Extensions and costs" } " · "
                a href="#worksheet" { "Migration worksheet" } " · " a href="#decision" { "Make the decision" }
            }
            section aria-labelledby="compare" {
                h2 id="compare" { "Compare the requirements that can rule an editor out" }
                p {
                    "Both projects provide macOS, Linux and Windows downloads, as their "
                    a href="https://zed.dev/docs/installation" { "Zed installation guide" } " and "
                    a href="https://github.com/lapce/lapce" { "Lapce repository" } " confirm. Platform availability is only the first check. "
                    "For example, Zed's local Linux application requires Vulkan 1.3 support. Verify the actual machine and driver you use before planning a migration. "
                    "That requirement does not establish that Lapce will run on hardware where Zed will not."
                }
                p class="small muted python-table-hint" { "Scroll sideways to compare both editors and the acceptance check." }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Zed and Lapce comparison table, scroll horizontally on small screens" {
                    table class="task-table" {
                        caption { "Documented capabilities, followed by a check on your own setup" }
                        thead { tr { th scope="col" { "Requirement" } th scope="col" { "Zed" } th scope="col" { "Lapce" } th scope="col" { "Check before switching" } } }
                        tbody {
                            tr { th scope="row" { "Rust language intelligence" } td { "Native rust-analyzer integration" } td { "rust-analyzer plugin" } td { "Navigate, diagnose and format your workspace" } }
                            tr { th scope="row" { "Python workflow" } td { "Native basedpyright and Ruff support" } td { "Verify the exact Python plugin and features you need" } td { "Correct environment, imports and diagnostics" } }
                            tr { th scope="row" { "Remote development" } td { "Local UI with an SSH-connected server" } td { "Built-in remote development" } td { "Open, edit and run a task on your intended host" } }
                            tr { th scope="row" { "Editor extensions" } td { "Zed extension system" } td { "WASI-based plugin system" } td { "Find a supported equivalent for every essential feature" } }
                            tr { th scope="row" { "Costs" } td { "Free editor; optional hosted services can cost extra" } td { "Open-source editor; Lapdev is a separate product" } td { "Separate editor needs from service subscriptions" } }
                        }
                    }
                }
                p {
                    "For Python in Lapce, verify the current plugin, environment selection, formatter and debugger against your project before switching."
                }
            }
            section aria-labelledby="languages" {
                h2 id="languages" { "Rust and Python: check the language workflow, not syntax colors" }
                p {
                    "Zed's " a href="https://zed.dev/docs/languages/rust" { "Rust documentation" }
                    " describes native rust-analyzer integration. Lapce's " a href="https://github.com/lapce/lapce-rust" { "official Rust plugin" }
                    " also uses rust-analyzer. That shared language server makes a blanket claim that only one is suitable for Rust hard to defend. "
                    "It does not prove that navigation, code actions, debugging or configuration behave identically in the two editor interfaces."
                }
                p {
                    "For a Rust workspace, start from a clean working tree and open the workspace root. Pick a symbol from another local module "
                    "and follow its definition. Make a harmless type error on a throwaway branch, check the diagnostic, then undo it. "
                    "Format a small change and inspect the diff. Run your normal test task. If breakpoints are part of your day, "
                    "confirm the debugger stops where you expect; a working language server alone does not settle that requirement."
                }
                p {
                    "For Python, " a href="https://zed.dev/docs/languages/python" { "Zed documents basedpyright and Ruff as native language services" }
                    ", with Python installed separately. Choose the project's intended Python toolchain, then check an import from that environment "
                    "and a deliberate type error. Use the same project when evaluating Lapce, verifying its current plugin and interpreter configuration "
                    "instead of assuming that a file opening successfully means the environment is correct."
                }
                p {
                    "Keep editor selection separate from changing your Python package manager or formatter rules. Our "
                    a href="/guides/rust-python-tools" { "guide to uv, Ruff and Polars" }
                    " separates those jobs. If a migration changes the editor, environment and lint rules together, a different diagnostic becomes "
                    "harder to explain. Hold the project configuration steady until you know which change you actually want."
                }
            }
            section aria-labelledby="remote" {
                h2 id="remote" { "SSH: a local launch does not prove the remote workflow" }
                p {
                    "Zed's " a href="https://zed.dev/docs/remote-development" { "remote-development documentation" }
                    " describes a local interface connected over SSH to a remote server, with language servers, tasks and terminals running remotely. "
                    "The " a href="https://lap.dev/lapce/" { "Lapce project page" }
                    " also documents built-in remote development. An SSH feature label is not evidence that your particular host, architecture, "
                    "shell, network policy and plugins will work without setup."
                }
                p {
                    "Use a non-sensitive test workspace on a host you are authorized to access. Verify an ordinary SSH connection first, then "
                    "connect through the editor and confirm which machine holds the files. Change a test file, reopen it, and run a harmless "
                    "project task remotely. Check that diagnostics refer to the remote environment, not an unrelated local toolchain. "
                    "Repeat after disconnecting and reconnecting so that a one-off successful connection is not the whole trial."
                }
                p {
                    "Server installation also matters on restricted networks. Zed documents both remote download and a local-download/upload "
                    "option for its matching server binary. Consult the current instructions rather than disabling a firewall or guessing a binary version. "
                    "For Lapce, check your release's remote setup against the intended host and repeat the same workspace tasks after reconnecting."
                }
            }
            section aria-labelledby="extensions" {
                h2 id="extensions" { "Extensions, modal editing and paid services are separate decisions" }
                p {
                    "An extension's useful behavior is the migration requirement, not its name. List what it does: attach a debugger, preview a format, "
                    "manage a notebook, or run a language-specific action. Then find and test the editor-native equivalent. "
                    a href="https://zed.dev/docs/extensions" { "Zed has its own extension system" }
                    "; Lapce's repository describes WASI-based plugins. Shared Language Server Protocol support does not establish that "
                    "an editor extension transfers unchanged. Keep your current editor for any essential workflow you cannot reproduce."
                }
                p {
                    a href="https://github.com/lapce/lapce" { "Lapce's repository" } " documents toggleable Vim-like modal editing and an Apache-2.0-licensed codebase. Those are concrete reasons "
                    "to try it if the editing model or source license matters to your project. They are not evidence that Zed lacks modal editing "
                    "or that Lapce uses less memory. Source modification and redistribution questions should start with the actual license files, "
                    "not a feature-comparison score."
                }
                p {
                    "If shared editing is a requirement, Zed's " a href="https://zed.dev/docs/collaboration/overview" { "collaboration documentation" }
                    " describes real-time work in a shared project and requires sign-in. That is a concrete reason to include Zed in a team trial. "
                    "Have an authorized collaborator check the actual joining, editing and leaving workflow before relying on it. "
                    "This does not establish that collaboration is paid, or that another editor cannot support collaboration through a different setup."
                }
                p {
                    "Zed's " a href="https://zed.dev/pricing" { "current pricing page" }
                    " separates its free editor from optional paid hosted AI services. Consider those services only if you need them; an external "
                    "AI provider may have its own bill. Lapce and Lapdev are different products, even though their pages share navigation. "
                    "Do not price a local Lapce trial using a Lapdev hosted-environment plan. This comparison does not require you to subscribe "
                    "to a service or send project code to an AI provider."
                }
            }
            section aria-labelledby="worksheet" {
                h2 id="worksheet" { "Use a pass, blocked or unverified migration worksheet" }
                p {
                    "Write the acceptance condition before trying an editor. A pass means the required operation worked on the stated setup. "
                    "Blocked means you reproduced a failure that prevents your work. Unverified means you did not complete the check. "
                    "Do not turn unverified into either a pass or an accusation that a feature does not exist."
                }
                figure class="python-layers" {
                    svg viewBox="0 0 380 330" width="100%" role="img" aria-labelledby="editor-gates-title editor-gates-desc" {
                        title id="editor-gates-title" { "Editor migration: requirements before preferences" }
                        desc id="editor-gates-desc" { "Check platform and required tasks first. Keep the existing editor if a required task is blocked or unverified. Compare preferences only after required tasks pass." }
                        @for (y, heading, detail) in [(4, "1. Can it run here?", "Check platform and graphics"), (94, "2. Can I do the work?", "Language, extensions and SSH"), (184, "3. Do I prefer it?", "Only after required tasks pass")] {
                            rect x="1" y=(y) width="378" height="72" rx="10" fill="var(--bg-soft)" stroke="var(--border-strong)" {}
                            text x="18" y=(y + 28) fill="var(--accent)" font-size="21" font-weight="700" { (heading) }
                            text x="18" y=(y + 54) fill="currentColor" font-size="17" { (detail) }
                        }
                        path d="M190 77 v15 m-5 -6 l5 6 l5 -6 M190 167 v15 m-5 -6 l5 6 l5 -6" fill="none" stroke="currentColor" stroke-width="2" {}
                        text x="18" y="284" fill="currentColor" font-size="15" { "Blocked or unverified? Keep your fallback." }
                        text x="18" y="314" fill="var(--fg-muted)" font-size="12" { "Built with Rust · October 2026" }
                    }
                    figcaption class="small muted" { "A preferred interface cannot compensate for a missing task you need every day." }
                }
                ol {
                    li { strong { "Record the setup. " } "Editor release, operating system, graphics driver where relevant, project revision, language tools, and local or remote execution. Do not mix a nightly build in one trial with an unspecified stable build in the other." }
                    li { strong { "Name the non-negotiables. " } "Use concrete actions: follow a cross-file symbol, see a known diagnostic, format without unrelated changes, stop at a breakpoint, or reopen a remote workspace. Separate required tasks from nice-to-have preferences." }
                    li { strong { "Run the same work. " } "Use the same test project and expected results in both editors. Save pass, blocked or unverified beside each task, plus the observation that justifies it. If you change settings to fix a failure, record the change and repeat the check." }
                    li { strong { "Make a reversible choice. " } "Switch one project first only if every required task passes. Keep your previous configuration and editor installed. If a task is blocked, continue with the working setup while investigating; if unverified, finish that check before committing to the migration." }
                }
                p { strong { "Copy this worksheet into your project notes. " } "Record each editor's release, OS, project revision and local or remote host type. Mark optional tasks as not required before the trial; every required task starts unverified." }
                p class="small muted python-table-hint" { "Scroll sideways for both editors' result columns." }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Editor migration worksheet, scroll horizontally on small screens" {
                    table class="task-table" {
                        caption { "Your acceptance record: replace unverified with a status and observation" }
                        thead { tr { th scope="col" { "Required task" } th scope="col" { "Expected result" } th scope="col" { "Zed: status + observation" } th scope="col" { "Lapce: status + observation" } } }
                        tbody {
                            @for (task, expected) in [
                                ("Cross-file navigation", "Opens the expected definition in this workspace"),
                                ("Known diagnostic", "Reports the deliberately introduced type error"),
                                ("Formatting", "Expected diff with no unrelated rule changes"),
                                ("Project test task", "Same command and environment; expected result"),
                                ("Required breakpoint", "Stops in the intended executable; mark not required if unused"),
                                ("Essential extension", "Your named behavior is reproduced"),
                                ("SSH reconnect", "Intended remote files, tools and task still work"),
                            ] {
                                tr { th scope="row" { (task) } td { (expected) } td { "Unverified — add observation" } td { "Unverified — add observation" } }
                            }
                        }
                    }
                }
            }
            section aria-labelledby="decision" {
                h2 id="decision" { "Which editor should you try first?" }
                p {
                    strong { "Start with Zed " }
                    "when its documented native Rust or Python setup matches your daily work and you want to evaluate that integration before managing "
                    "extra language plugins. The advantage here is a documented starting configuration, not a guarantee that every debugger, extension "
                    "or remote environment will fit. An essential incompatible workflow is enough reason to keep your existing editor."
                }
                p {
                    strong { "Start with Lapce " }
                    "when you specifically want to explore its toggleable modal interface, Rust plugin workflow, or Apache-licensed source. "
                    "Give it the same real-workspace checks rather than dismissing it for having a different extension model. Conversely, "
                    "a source-license preference alone is not a reason to abandon a working development environment."
                }
                p {
                    strong { "Keep your current editor " }
                    "if neither candidate passes the tasks you need. You can still explore other "
                    a href="/categories/developer-tools" { "Rust developer tools by task" }
                    " without moving your editor. When ready, open the " a href="/projects/zed" { "Zed project details" }
                    " or " a href="/projects/lapce" { "Lapce project details" } " for the official download and source links."
                }
            }
            section aria-labelledby="method" {
                h2 id="method" { "Sources and limits" }
                p {
                    "Official documentation and repositories were checked on October 13, 2026. The recommendations are editorial judgments "
                    "about workflow fit. We did not benchmark startup time, memory use, battery life or editing latency, and did not perform the "
                    "worksheet in either editor. Unverified Lapce Python details are not evidence that Python is unsupported. Product features can change; "
                    "follow the linked official instructions for your release. No overall numerical score is assigned."
                }
            }
            section aria-labelledby="faq" {
                h2 id="faq" { "Common editor-choice questions" }
                @for (q, a) in faqs { h3 { (q) } p { (a) } }
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string())) }
    }
}
