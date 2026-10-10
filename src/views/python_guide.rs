//! Static editorial content: rendered and compressed once through the page cache.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/rust-python-tools");
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@graph": [
            {"@type": "WebPage", "@id": url, "url": url,
             "name": "Rust-powered Python tools: uv, Ruff and Polars", "datePublished": "2026-10-10", "dateModified": "2026-10-10",
             "author": {"@type": "Organization", "name": "Built with Rust", "url": site_url}},
            {"@type": "BreadcrumbList", "itemListElement": [
                {"@type": "ListItem", "position": 1, "name": "Projects", "item": site_url},
                {"@type": "ListItem", "position": 2, "name": "Rust-powered Python tools", "item": url}
            ]},
            {"@type": "FAQPage", "mainEntity": [
                {"@type": "Question", "name": "Can I use uv, Ruff and Polars together?", "acceptedAnswer": {"@type": "Answer", "text": "Yes. uv manages the Python project environment, Ruff checks and formats Python source, and Polars handles dataframe operations. You can also adopt each independently."}},
                {"@type": "Question", "name": "Does Ruff replace a type checker?", "acceptedAnswer": {"@type": "Answer", "text": "No. Ruff is a linter and formatter, not a type checker. Keep type checking and tests as separate parts of your workflow."}},
                {"@type": "Question", "name": "Is Polars a drop-in replacement for pandas?", "acceptedAnswer": {"@type": "Answer", "text": "No. Polars has a different API and execution model, including expressions and no pandas-style row index. Migrate and validate a representative transformation before expanding its use."}}
            ]}
        ]
    });
    html! {
        article class="python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / Rust-powered Python tools" }
            h1 { "Rust-powered Python tools: uv, Ruff and Polars" }
            p class="guide-lead" {
                "Rust-powered Python tools solve different problems: uv manages project environments and dependencies, Ruff lints and formats Python source, and Polars processes dataframe workloads. Choose the layer causing friction first. You can use these tools together or independently; adopting a tool written in Rust does not turn your Python application into Rust."
            }
            p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-10-10" { "October 10, 2026" } }
            p {
                "A new Python project can start with all three, but an existing project does not need to migrate all three layers at once. "
                "This guide compares their jobs, the changes each introduces, and a small trial you can use to decide whether to keep it. "
                "The examples come from the Built with Rust catalog; this is a selection guide, not a benchmark or a ranking of every Python tool."
            }
            section aria-labelledby="choose-layer" {
                h2 id="choose-layer" { "Choose the job, not the implementation language" }
                p {
                    "Start by naming the work you want to improve. Recreating an environment, reviewing lint failures and running a data transformation are separate tasks. "
                    "A tool can improve one without affecting the others. If your application spends its time waiting for an external service, "
                    "changing its formatter is not a runtime optimization. Keep the question narrow enough that you can check the result."
                }
                figure class="python-layers" {
                    svg viewBox="0 0 380 300" width="100%" role="img" aria-labelledby="python-layers-title python-layers-desc" {
                        title id="python-layers-title" { "Three independently adoptable Python tooling layers" }
                        desc id="python-layers-desc" { "uv manages the environment. Ruff checks source code. Polars processes data. These are separate choices, not required steps in a pipeline." }
                        @for (y, name, task) in [(10, "uv", "Environment and dependencies"), (100, "Ruff", "Source checks and formatting"), (190, "Polars", "Dataframe operations")] {
                            rect x="1" y=(y) width="378" height="76" rx="10" fill="var(--bg-soft)" stroke="var(--border-strong)" {}
                            text x="20" y=(y + 29) fill="var(--accent)" font-size="22" font-weight="700" { (name) }
                            text x="20" y=(y + 56) fill="currentColor" font-size="17" { (task) }
                        }
                        text x="20" y="292" fill="var(--fg-muted)" font-size="12" { "Built with Rust · October 2026" }
                    }
                    figcaption class="small muted" { "Pick one layer to change. The other two can stay as they are." }
                }
                p class="small muted python-table-hint" { "Scroll the table sideways to see the trial checks." }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Tool selection table, scroll horizontally on small screens" {
                    table class="task-table" {
                        caption { "A first trial and a keep-or-stop check for each tool" }
                        thead { tr { th scope="col" { "Your problem" } th scope="col" { "Try" } th scope="col" { "Leave unchanged" } th scope="col" { "Check before keeping it" } } }
                        tbody {
                            tr { th scope="row" { "Environment setup is awkward" } td { a href="/projects/uv" { "uv" } ": recreate one project environment" } td { "Application code, lint rules and dataframe library" } td { "Dependencies resolve; tests and CI still run" } }
                            tr { th scope="row" { "Code checks are slow or fragmented" } td { a href="/projects/ruff" { "Ruff" } ": inspect lint and formatting reports" } td { "Package manager, type checker and runtime code" } td { "Required rules are covered; formatting diffs are acceptable" } }
                            tr { th scope="row" { "A data transformation needs attention" } td { a href="/projects/polars" { "Polars" } ": port one representative transformation" } td { "Unrelated pipelines and development tooling" } td { "Outputs match; time and memory suit the workload" } }
                        }
                    }
                }
            }
            section aria-labelledby="uv" {
                h2 id="uv" { "uv: change how the project environment is managed" }
                p {
                    a href="/projects/uv" { "uv" } " is Astral’s Python package and project manager, written in Rust. "
                    "It manages dependencies, virtual environments and Python versions. It can also install and run packaged command-line tools. "
                    "That puts it around your application: it prepares the environment in which Python runs. "
                    "See the " a href="https://docs.astral.sh/uv/" { "official uv overview" } " for its supported workflows."
                }
                p {
                    "In uv’s project workflow, pyproject.toml describes the project and its requirements, uv.lock records resolved dependencies, "
                    "and a project environment holds installed packages. The " a href="https://docs.astral.sh/uv/guides/projects/" { "project guide" }
                    " explains how these pieces work together. This is distinct from using uv’s pip interface to manage an existing environment. "
                    "Choose the workflow deliberately instead of mixing instructions from both and assuming they maintain the same project state."
                }
                p {
                    "For an existing repository, trial uv on a branch and in a fresh environment. Confirm the Python version, dependency sources "
                    "and test command before comparing setup times. Check private package indexes and CI explicitly: "
                    a href="https://docs.astral.sh/uv/pip/compatibility/" { "uv’s pip compatibility documentation" }
                    " describes differences from pip, including configuration handling. Keep the current workflow if a required integration "
                    "does not yet work or the change costs more than the problem it solves. You can evaluate Ruff without changing package managers."
                }
            }
            section aria-labelledby="ruff" {
                h2 id="ruff" { "Ruff: change feedback on Python source" }
                p {
                    a href="/projects/ruff" { "Ruff" } " is a Python linter and formatter written in Rust. Linting reports selected code-quality "
                    "issues; formatting applies a consistent layout. Neither job resolves the application’s dependencies or executes its dataframe queries. "
                    "The " a href="https://docs.astral.sh/ruff/" { "Ruff overview" } " describes the two capabilities, which you can adopt separately."
                }
                p {
                    "Begin with reports, not an automatic rewrite. Check existing editor, hook and auto-fix settings first. Inventory the lint rules your team relies on, compare their coverage, "
                    "and inspect proposed changes on a branch. Ruff distinguishes safe and unsafe fixes; that distinction matters when deciding "
                    "which changes to automate. Its " a href="https://docs.astral.sh/ruff/linter/" { "linter documentation" }
                    " is the source for rule selection and fix behavior. A cleaner report is not enough if an important check disappeared during migration."
                }
                p {
                    "Treat formatting as a separate decision. Ruff aims for compatibility with Black’s style but documents intentional differences, "
                    "and its maintainers discourage alternating the two formatters on the same codebase. Review a formatting-only diff before "
                    "combining it with a functional change. Read the " a href="https://docs.astral.sh/ruff/formatter/" { "formatter documentation" }
                    " and retain your type checker: " a href="https://docs.astral.sh/ruff/faq/" { "Ruff is not a type checker" }
                    ". Keep running existing tests and add focused checks for changed behavior."
                }
            }
            section aria-labelledby="polars" {
                h2 id="polars" { "Polars: change the dataframe work itself" }
                p {
                    a href="/projects/polars" { "Polars" } " is a dataframe library with a Rust core and a Python interface. "
                    "Unlike uv and Ruff, it can be part of the code that processes your application’s data. "
                    "Its " a href="https://docs.pola.rs/" { "user guide" } " introduces expressions for operations such as selecting, filtering "
                    "and aggregating columns. Choose it because that model fits a data task, not because every project should have a Rust-based library."
                }
                p {
                    "Polars supports eager work and lazy query plans. Lazy execution lets the engine optimize a plan before running it; "
                    "the " a href="https://docs.pola.rs/user-guide/concepts/lazy-api/" { "lazy API guide" } " explains that distinction. "
                    "This is not a promise that every workload becomes faster or fits in memory. If the real bottleneck is a Python callback, "
                    "review whether a native expression can represent the operation. The "
                    a href="https://docs.pola.rs/user-guide/expressions/user-defined-python-functions/" { "user-defined function guide" }
                    " explains callback tradeoffs; custom Python functions can still be useful."
                }
                p {
                    "A pandas migration is more than changing an import. Polars has different API and type semantics and no pandas-style row index. "
                    "Use the " a href="https://docs.pola.rs/user-guide/migration/pandas/" { "official pandas migration guide" }
                    " to plan a trial. Port one representative transformation, then compare values, null handling, data types, joins and output ordering "
                    "where your application relies on them. Retain pandas when the existing pipeline meets your needs or downstream integrations "
                    "make the migration unjustified. Adopting uv or Ruff does not require this rewrite."
                }
            }
            section aria-labelledby="first-change" {
                h2 id="first-change" { "Make the first change small enough to judge" }
                p {
                    "Use the table as a worksheet, not an instruction to install everything. Write down one current frustration and the result "
                    "that would justify a switch. For environment management, that might be a reliable fresh setup in CI. For source checks, "
                    "it might be shorter feedback while retaining required rules. For a data pipeline, it might be meeting a runtime or memory "
                    "limit without changing the result. These are proposed acceptance checks, not improvements measured by this directory."
                }
                ol {
                    li { strong { "Record the working baseline. " } "Keep the current dependency files and configuration in version control. Record the command or process you are replacing and the inputs it uses." }
                    li { strong { "Change one layer on a branch. " } "Avoid a package-manager migration, a repository-wide formatting diff and a dataframe rewrite in the same review. Separate changes make surprises easier to trace and reverse." }
                    li { strong { "Check correctness before speed. " } "Run the existing tests, inspect source changes and compare data outputs. Add a focused check where the current suite does not cover the behavior you are changing." }
                    li { strong { "Measure the relevant task. " } "Compare equivalent inputs and record tool versions, hardware and cache state. A warm installation and a cold installation answer different questions; neither is an application runtime benchmark." }
                    li { strong { "Keep or stop explicitly. " } "Adopt the change if it meets the agreed check. Otherwise keep the working setup and record the incompatibility or missing benefit, rather than migrating the next layer to justify the first." }
                }
                p {
                    "For a new project, the same boundaries still help. Decide whether you need dataframe processing at all before choosing Polars. "
                    "Then choose environment management and source checks on their own merits. A small service can use uv and Ruff without Polars; "
                    "a data application can use Polars with its existing tools. There is no combined speed multiplier to calculate from three "
                    "benchmarks that measure different jobs."
                }
            }
            section aria-labelledby="questions" {
                h2 id="questions" { "Common selection questions" }
                h3 { "Can I use uv, Ruff and Polars together?" }
                p { "Yes. uv manages the Python project environment, Ruff checks and formats Python source, and Polars handles dataframe operations. You can also adopt each independently." }
                h3 { "Does Ruff replace a type checker?" }
                p { "No. Ruff is a linter and formatter, not a type checker. Keep type checking and tests as separate parts of your workflow." }
                h3 { "Is Polars a drop-in replacement for pandas?" }
                p { "No. Polars has a different API and execution model, including expressions and no pandas-style row index. Migrate and validate a representative transformation before expanding its use." }
            }
            section aria-labelledby="next-tool" {
                h2 id="next-tool" { "Explore the tool for your next change" }
                p {
                    "Open the " a href="/projects/uv" { "uv listing" } ", " a href="/projects/ruff" { "Ruff listing" } " or "
                    a href="/projects/polars" { "Polars listing" } " for its website and source repository. "
                    "For adjacent tasks, browse " a href="/categories/developer-tools" { "Rust-built developer tools" } " and "
                    a href="/categories/data-and-ai" { "data and AI projects" } ". Check current installation requirements upstream before switching."
                }
                p class="small muted" {
                    "Scope: selected catalog projects, checked against their official documentation on October 10, 2026. "
                    "The trial checklist is Built with Rust’s editorial recommendation, not a vendor benchmark. "
                    "Directory star counts are stored snapshots, not live totals."
                }
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
