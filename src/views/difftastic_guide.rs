//! Versioned editorial comparison and disposable review exercise.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/difftastic-vs-git-diff");
    let faqs = [
        (
            "Can Difftastic replace git diff?",
            "Use Difftastic as an optional structural review view. Keep native Git diff for textual changes, patch export and tools that expect unified diffs. Difftastic does not replace Git's repository, staging or merge operations.",
        ),
        (
            "Does no syntactic changes mean a change is safe?",
            "No. It describes the compared inputs under Difftastic's language detection and parsing, not application behavior. Review the textual diff and run the checks appropriate to the change.",
        ),
        (
            "Does Difftastic ignore all whitespace?",
            "No. The supplied Python fixture changes block membership by changing indentation, and Difftastic 0.71.0 reports a difference. Formatting-only JavaScript in the same exercise reports no syntactic changes.",
        ),
    ];
    let schema = serde_json::json!({
        "@context": "https://schema.org", "@graph": [
            {"@type": "Article", "@id": url, "url": url,
             "headline": "Difftastic vs git diff: keep both views",
             "datePublished": "2026-10-26T21:38:00Z", "dateModified": "2026-10-26T21:38:00Z",
             "author": {"@type": "Organization", "name": "Built with Rust editorial", "url": site_url}},
            {"@type": "BreadcrumbList", "itemListElement": [
                {"@type": "ListItem", "position": 1, "name": "Projects", "item": site_url},
                {"@type": "ListItem", "position": 2, "name": "Guides", "item": format!("{site_url}/guides")},
                {"@type": "ListItem", "position": 3, "name": "Difftastic vs git diff", "item": url}
            ]},
            {"@type": "FAQPage", "mainEntity": faqs.iter().map(|(q,a)| serde_json::json!({
                "@type": "Question", "name": q, "acceptedAnswer": {"@type": "Answer", "text": a}
            })).collect::<Vec<_>>()}
        ]
    });
    html! {
        article class="python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" {
                a href="/" { "Projects" } " / " a href="/guides" { "Guides" } " / Difftastic vs git diff"
            }
            h1 { "Difftastic vs git diff: keep both views" }
            p class="guide-lead" {
                "Choose Difftastic when you need to read a code change through its syntax, especially when formatting obscures a smaller edit. "
                "Keep native git diff when exact textual changes or an applicable patch are the job. Difftastic vs git diff is a choice of review view, not a replacement for version control or tests."
            }
            p class="small muted" {
                "By Built with Rust editorial · Published " time datetime="2026-10-26T21:38:00Z" { "October 27, 2026" }
            }
            p {
                a href="/projects/difftastic" { "Difftastic" } " is a Rust-built command-line tool, invoked as " code { "difft" } ". "
                "It compares parsed structure rather than treating every line break as a boundary. This guide is for developers deciding whether to add it to an existing Git workflow. "
                "The useful test is not whether a screenshot looks cleaner. It is whether you can identify the edit, notice when structural comparison is unavailable, and still produce the normal patch your workflow needs."
            }
            nav aria-label="On this page" {
                ol {
                    li { a href="#decision" { "Choose the view by the task" } }
                    li { a href="#exercise" { "Run the four-case comparison" } }
                    li { a href="#git" { "Try Difftastic without global configuration" } }
                    li { a href="#patch" { "Keep the patch path separate" } }
                    li { a href="#worksheet" { "Use a review acceptance worksheet" } }
                    li { a href="#limits" { "Know the limits" } }
                    li { a href="#questions" { "Common questions" } }
                }
            }
            section aria-labelledby="decision" {
                h2 id="decision" { "Difftastic vs git diff: choose by the task" }
                p {
                    "A structural diff and a textual diff answer different questions. Difftastic helps you inspect changes to syntax; Git's native patch view preserves a textual representation suitable for the patch workflow. "
                    "Treat the first as an additional reading lens. Do not make its output the input to a command that expects a unified diff. "
                    "The project's " a href="https://github.com/Wilfred/difftastic/blob/b7d119e90ac9f972f03f69508da765dacd302c0c/README.md#non-goals" { "documented non-goals" }
                    " explicitly exclude generating applicable patches and performing merges."
                }
                p class="small muted python-table-hint" { "Scroll sideways, or focus the table and use the arrow keys." }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Difftastic and Git task comparison, scroll horizontally" {
                    table class="task-table" {
                        caption { "Choose the output you need, not a universal winner" }
                        thead { tr { th scope="col" { "Your task" } th scope="col" { "Start with" } th scope="col" { "Keep this check" } } }
                        tbody {
                            tr { th scope="row" { "Read code mixed with reformatting" } td { "Difftastic structural view" } td { "Check the detected language and inspect the text too" } }
                            tr { th scope="row" { "Inspect exact layout or whitespace edits" } td { "Native git diff" } td { "Do not infer unchanged text from unchanged syntax" } }
                            tr { th scope="row" { "Export a patch for git apply" } td { "Native git diff with external diff and textconv disabled" } td { "Validate against the intended base" } }
                            tr { th scope="row" { "Review malformed or unsupported syntax" } td { "Read Difftastic's fallback header or use native Git" } td { "Do not mistake Text fallback for structural analysis" } }
                            tr { th scope="row" { "Stage selected changes or resolve a merge" } td { "Git's staging or merge workflow" } td { "A clearer view does not perform either operation" } }
                        }
                    }
                }
                p {
                    "If colors are the only improvement you want, first try Git's own word-level display or a pager. A different display is not automatically a different comparison algorithm. "
                    "You can evaluate " code { "git diff --word-diff" } " alongside Difftastic on the same edit before adding another tool. "
                    "For a separate terminal interface to staging and repository operations, see " a href="/projects/gitui" { "GitUI" }
                    "; that is a different job from producing a structural diff."
                }
            }
            section aria-labelledby="exercise" {
                h2 id="exercise" { "A reproducible Difftastic vs git diff exercise" }
                p {
                    "The downloadable exercise creates four pairs of tiny files in an automatically cleaned temporary directory. "
                    "It runs both tools against the same pair, prints their output and checks the expected exit codes. It then creates a disposable Git index, tries the one-command external diff and verifies a native patch round-trip. "
                    "No existing repository, Git configuration or personal files are changed. It makes no network requests and does not execute the sample JavaScript or Python."
                }
                p {
                    "Install Git, Python 3 and Difftastic using the "
                    a href="https://difftastic.wilfred.me.uk/installation.html" { "official installation guide" }
                    ". Download " a href="/assets/examples/difftastic-review-v1/lab.py" download="lab.py" { "lab.py" }
                    ", read it, then run it from any folder:"
                }
                pre tabindex="0" aria-label="Command example, scroll horizontally" { code { "git --version\ndifft --version\npython3 lab.py" } }
                p {
                    "We executed this exercise with Difftastic 0.71.0 (revision b7d119e), Git 2.39.5 and Python 3.11.2 on Linux on October 27, 2026. "
                    "These are observed results for those inputs and versions, not a benchmark or a cross-platform certification. The script stops if a result differs; inspect the output instead of changing the expected result merely to make it pass."
                }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Observed four-case comparison results, scroll horizontally" {
                    table class="task-table" {
                        caption { "Observed results with explicit difference exit codes enabled" }
                        thead { tr { th scope="col" { "Case" } th scope="col" { "Native Git exit" } th scope="col" { "Difftastic exit and display" } } }
                        tbody {
                            tr { th scope="row" { "JavaScript object reflow, same values" } td { "1: text differs" } td { "0: No syntactic changes" } }
                            tr { th scope="row" { "Same reflow, retries changes from 3 to 4" } td { "1: text differs" } td { "1: JavaScript change" } }
                            tr { th scope="row" { "Python call moves outside an if block" } td { "1: text differs" } td { "1: Python change" } }
                            tr { th scope="row" { "JavaScript initializer becomes invalid" } td { "1: text differs" } td { "1: Text, with a parse-error explanation" } }
                        }
                    }
                }
                p {
                    "The first pair changes one line into several lines without changing the object's values. The second pair adds one value edit to that layout change. "
                    "Comparing them makes the distinction visible: a large textual replacement can contain either no detected syntax change or a real token change. "
                    "The Python pair prevents the wrong generalization that indentation is always disposable. The malformed JavaScript pair makes fallback an observable event rather than a footnote."
                }
                p {
                    "The exit-code comparison is deliberate. The script requests " code { "difft --exit-code" } "; without that option you must not read ordinary successful command completion as proof that inputs match. "
                    "Git's " code { "--no-index" } " comparison implies difference exit codes. A result of 1 here is an expected difference, not a crashed command. "
                    "The " a href="https://git-scm.com/docs/git-diff#Documentation/git-diff.txt---no-index" { "Git diff reference" }
                    " documents that distinction. Do not reuse this little exercise as a generic production change detector."
                }
            }
            section aria-labelledby="git" {
                h2 id="git" { "Try Difftastic without changing your global Git setup" }
                p {
                    "Start on one tracked source file with an unstaged edit. This command opts into the external diff for that invocation only; it does not write a global preference. "
                    "Replace the sample path with a file in your repository. The command follows the project's "
                    a href="https://github.com/Wilfred/difftastic/blob/b7d119e90ac9f972f03f69508da765dacd302c0c/manual/src/git.md" { "Git integration instructions" } ":"
                }
                pre tabindex="0" aria-label="Command example, scroll horizontally" { code { "git --no-pager -c diff.external=difft diff --ext-diff -- src/example.js" } }
                p {
                    "Keep comparison scope identical when switching views. The command above compares the working tree with the index. "
                    "For staged edits, add " code { "--cached" } " before the path separator in both commands. If you compare commits, use the same revisions in both. "
                    "A clearer result from a different base tells you nothing about which view helped. New untracked files are also a separate review task; do not assume an ordinary unstaged diff includes them."
                }
                p {
                    "Check your Git version before adopting the integration broadly. Difftastic's integration documentation warns that Git 2.43.1 and earlier can crash with external diffs when file modes change. "
                    "Our Git 2.39.5 exercise covers regular-file content changes, not mode changes, renames, submodules or a repository-wide rollout. "
                    "Use a newer Git and verify the file types you rely on before making Difftastic your default. If the one-off trial is not useful, omit the command; there is no global setting to undo."
                }
            }
            section aria-labelledby="patch" {
                h2 id="patch" { "Keep a native patch path you can verify" }
                p {
                    "Use the structural display to read; use Git's native output to transfer a change. Explicitly disable external diff and text conversion when exporting a patch. "
                    "The following example exports unstaged changes to one tracked file. Save the patch outside the working tree so it does not become another file to review:"
                }
                pre tabindex="0" aria-label="Command example, scroll horizontally" { code { "git --no-pager diff --no-ext-diff --no-textconv --binary -- src/example.js > ../review.patch" } }
                p {
                    "Validate it in a disposable checkout at the exact pre-change base with " code { "git apply --check ../review.patch" }
                    ". That command checks applicability without applying the patch. Do not expect a patch to apply cleanly on top of the changes it already contains. "
                    "The exercise avoids that mistake: it saves the patch, restores the temporary file's before-content, checks applicability, applies it, and compares the resulting content with the expected after-file. "
                    "The observed round-trip passed. See the " a href="https://git-scm.com/docs/git-apply" { "git apply reference" } " for the check's scope."
                }
                p {
                    "Patch applicability is not correctness. A patch can apply and still introduce a bug. Equally, Difftastic can correctly identify a changed value without knowing whether that value is appropriate. "
                    "Keep your normal tests, type checks and review of the requirement. This split lets you adopt a Rust-built review tool without changing the interchange format expected by colleagues or automation."
                }
            }
            section aria-labelledby="worksheet" {
                h2 id="worksheet" { "A review acceptance worksheet" }
                p {
                    "Use this worksheet on one representative change before changing defaults. Record pass, blocked or unverified for each row, plus the file and tool versions. "
                    "Unverified means you have not checked it; it is not a pass. This is an editorial evaluation framework, not a measured improvement in review accuracy."
                }
                ol {
                    li { strong { "Scope matches. " } "Both views compare the same tracked files and the same working-tree, staged or commit base. Review untracked files separately." }
                    li { strong { "The edit remains visible. " } "Identify the changed token in the mixed layout/value case. Then inspect a representative edit from your own language and build setup." }
                    li { strong { "Fallback is recognized. " } "Read the file header. If it says Text or reports parser errors, record that structural analysis was unavailable; do not describe that result as a syntax-aware success." }
                    li { strong { "Text and patch survive. " } "Read the native diff too. Export a patch with external diff and textconv disabled, and check it against a disposable copy of the intended base." }
                    li { strong { "The actual requirement is checked. " } "Run the relevant application checks independently. Keep the old workflow if required file types are blocked or the new view obscures changes you need to see." }
                }
                p {
                    "Choose Difftastic if the structural view helps you understand real edits and the remaining rows pass. Keep native Git as your main view if exact text, unsupported files or patch-oriented tooling dominates the job. "
                    "Using both selectively is a valid outcome. Browse other " a href="/categories/developer-tools" { "Rust developer tools" }
                    " by the task they perform, or " a href="/submit" { "submit a Rust-built tool" } " that deserves a documented trial."
                }
            }
            section aria-labelledby="limits" {
                h2 id="limits" { "Syntax is not semantics, and fallback is not failure" }
                p {
                    "Difftastic compares the files it receives using language detection and parsers. Its "
                    a href="https://github.com/Wilfred/difftastic/blob/b7d119e90ac9f972f03f69508da765dacd302c0c/manual/src/tricky_cases.md" { "tricky-cases documentation" }
                    " explains why invalid or unsupported syntax can force a textual comparison. File-size and graph limits can also trigger a line-oriented fallback. "
                    "That is a boundary to recognize, not evidence that the tool silently performed the same structural analysis on every input."
                }
                p {
                    "A message saying no syntactic changes is narrower than saying behavior is unchanged. It does not evaluate your dependencies, runtime inputs or the correctness of your requirement. "
                    "Even " code { "--check-only --exit-code" } " is a syntax-oriented comparison, not a test suite. The "
                    a href="https://github.com/Wilfred/difftastic/blob/b7d119e90ac9f972f03f69508da765dacd302c0c/src/options.rs" { "release's command-line definitions" }
                    " describe that contract. Never use a green syntax comparison as sole permission to merge."
                }
                p {
                    "We did not measure runtime, memory, reviewer accuracy or time saved. The project documents performance and memory limitations on large changes; your largest generated files need their own trial. "
                    "We also did not verify every parser, operating system or Git integration. The floating online manual can describe development features: this guide's observed results refer specifically to "
                    a href="https://github.com/Wilfred/difftastic/releases/tag/0.71.0" { "release 0.71.0" } "."
                }
            }
            section aria-labelledby="questions" {
                h2 id="questions" { "Common questions" }
                @for (question, answer) in &faqs {
                    h3 { (question) }
                    p { (answer) }
                }
            }
            script type="application/ld+json" { (PreEscaped(schema.to_string())) }
        }
    }
}
