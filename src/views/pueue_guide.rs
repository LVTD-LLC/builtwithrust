//! Native, cached guide with a disposable task-outcome exercise.
use maud::{Markup, PreEscaped, html};

/// A task-specific route into the existing guide, not another tutorial.
pub fn discovery(site_url: &str, projects: &[crate::db::Project]) -> Markup {
    if !projects.iter().any(|p| p.slug == "pueue") {
        return html! {};
    }
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{site_url}/categories/productivity#command-queue-workflows"),
        "name": "Queue shell work, then verify the result",
        "dateModified": "2026-11-17",
        "isPartOf": { "@type": "CollectionPage", "@id": format!("{site_url}/categories/productivity") }
    });
    html! {
        section id="command-queue-workflows" class="task-guide" aria-labelledby="command-queue-title" {
            h2 id="command-queue-title" { "Queue shell work, then verify the result" }
            p class="guide-intro" {
                "Evaluate Pueue when your work is already a set of shell commands and you need to queue their execution. "
                "Keep three checks separate: the queue accepted the command, the selected task succeeded, and its output is correct. "
                "A finished task alone does not answer all three."
            }
            nav class="workflow-guides" aria-label="Shell command queue evaluation" {
                a href="/guides/pueue-task-queue" {
                    strong { "Check task completion, success and output →" }
                    span { "Six disposable Linux tasks contrast completion, process success, dependencies and expected files. Inspect the script and prerequisites before running it." }
                }
                a href="/projects/pueue" {
                    strong { "Inspect Pueue’s project and upstream links →" }
                    span { "Start with the tool’s scope and source before choosing a shell-command queue. This is a single-user shell workflow, not an application job service." }
                }
            }
            p class="guide-note" {
                "Need command recall or file navigation instead? Use the "
                a href="/guides/rust-terminal-tools" { "terminal tools by task guide" }
                ". The queue exercise is a controlled functional check, not a speed benchmark or a guarantee about your commands. "
                "Source: " a href="https://github.com/Nukesor/pueue/blob/v4.0.4/README.md" { "Pueue 4.0.4 scope" }
                "; links checked " time datetime="2026-11-17" { "November 17, 2026" } "."
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/pueue-task-queue");
    let wait_source = "https://github.com/Nukesor/pueue/blob/v4.0.4/pueue/src/client/commands/wait.rs";
    let cli_source = "https://github.com/Nukesor/pueue/blob/v4.0.4/pueue/src/client/cli.rs";
    let faqs = [
        (
            "Does pueue wait fail when a task fails?",
            "In Pueue 4.0.4, ordinary wait targets completion, including failed tasks. Use wait --status success with the actual task ID to check success. In our exercise, a task exited 7, ordinary wait returned 0, and success-wait returned 1. Keep the task record: an empty selection can also return success.",
        ),
        (
            "Does --after guarantee a dependent command cannot run?",
            "Normal Pueue scheduling requires successful dependencies. Our failed-prerequisite case left the dependent marker absent and recorded DependencyFailed. Explicitly force-starting a task can bypass dependencies, however. A paused group can also defer failure propagation. Use the normal scheduler and inspect the selected records rather than treating --after as an authorization barrier.",
        ),
        (
            "Why can a failing pipeline look successful in Pueue?",
            "Pueue observes the command process, not the meaning of each output. In Bash, a pipeline normally reports its last command's status. Our false-piped-to-cat script continued and produced a marker. A straight-line Bash script using set -e -o pipefail stopped instead. Confirm the intended artifact as well as the task result.",
        ),
    ];
    let schema = serde_json::json!({
        "@context":"https://schema.org", "@graph":[
            {"@type":"Article","@id":url,"url":url,"headline":"Pueue task queue: verify success",
             "datePublished":"2026-11-17","dateModified":"2026-11-17",
             "author":{"@type":"Organization","name":"Built with Rust editorial","url":site_url}},
            {"@type":"HowTo","name":"Check Pueue task outcomes in an isolated queue","step":[
                {"@type":"HowToStep","name":"Prepare pinned binaries","text":"Obtain trusted Pueue and pueued 4.0.4 binaries and inspect the downloaded Python exercise.","url":format!("{url}#prepare")},
                {"@type":"HowToStep","name":"Run the private exercise","text":"Run lab.py with absolute binary paths. It starts a temporary daemon and checks six synthetic tasks.","url":format!("{url}#run")},
                {"@type":"HowToStep","name":"Compare outcomes and artifacts","text":"Check task IDs, success-wait results, task records and expected files against the results table.","url":format!("{url}#results")}
            ]},
            {"@type":"BreadcrumbList","itemListElement":[
                {"@type":"ListItem","position":1,"name":"Projects","item":site_url},
                {"@type":"ListItem","position":2,"name":"Guides","item":format!("{site_url}/guides")},
                {"@type":"ListItem","position":3,"name":"Pueue task queue","item":url}]},
            {"@type":"FAQPage","mainEntity":faqs.iter().map(|(q,a)|serde_json::json!({
                "@type":"Question","name":q,"acceptedAnswer":{"@type":"Answer","text":a}
            })).collect::<Vec<_>>()}
        ]
    });
    html! {
        article class="python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / " a href="/guides" { "Guides" } " / Pueue task queue" }
            header {
                h1 { "Pueue task queue: verify success" }
                p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-11-17" { "November 17, 2026" }
                    " · Sources and exercise checked November 17, 2026" }
            }
            p class="guide-lead" {
                "Use the Pueue task queue when you want to line up shell commands and inspect their outcomes. "
                "A successful enqueue is not a successful command. Capture the task ID, wait for success rather than completion, "
                "and check the output you actually need before letting the next operation depend on it."
            }
            p {
                "Pueue is a Rust command-line task manager with a separate daemon, pueued, that executes queued work. "
                "Start with the " a href="/projects/pueue" { "Pueue project entry" } " for upstream links, or browse "
                a href="/categories/productivity" { "Rust productivity applications" } " for other jobs. "
                "If you need a terminal window, shell or history search instead, use the "
                a href="/guides/rust-terminal-tools" { "terminal workflow guide" } "."
            }
            p {
                "We ran six synthetic tasks using Pueue and pueued 4.0.4, Python 3.11.2 and Bash 5.2.15 on Linux. "
                "The exercise produced seven observations, including two different waits on the same failed task. "
                "It checks file state as well as process results. This is a functional example, not a throughput benchmark or a durability test."
            }
            nav class="guide-toc" aria-label="On this page" {
                ol {
                    li { a href="#contract" { "Separate three success boundaries" } }
                    li { a href="#prepare" { "Prepare an isolated trial" } }
                    li { a href="#run" { "Run the six-task exercise" } }
                    li { a href="#results" { "Interpret results and files" } }
                    li { a href="#handoff" { "Write a task acceptance contract" } }
                    li { a href="#limits" { "Know when Pueue is the wrong layer" } }
                    li { a href="#questions" { "Common questions" } }
                }
            }
            section aria-labelledby="contract" {
                h2 id="contract" { "Pueue task queue: separate acceptance from success" }
                p {
                    "Treat a queued operation as three decisions. First, did the daemon accept the command and return the intended task ID? "
                    "Second, did that task finish with a successful process result? Third, does its output satisfy the requirement? "
                    "These questions need different evidence. A task ID answers the first; a success result answers the second; an artifact assertion answers the third."
                }
                p {
                    "In the trial, a script containing exit 7 was accepted: the add client returned 0. "
                    "An ordinary wait on that ID also returned 0 after the script finished. A wait with the success target returned 1, "
                    "while the stored task result retained the original failure code, 7. "
                    "The wait client's code is a gate, not a copy of the child's exit code. "
                    "This agrees with the " a href=(wait_source) { "pinned wait implementation" } "."
                }
                pre { code { "id=$(pueue add --print-task-id -- 'your-command')\npueue wait --status success \"$id\"\npueue log \"$id\"" } }
                p {
                    "That sketch assumes an already configured queue and a real command you have inspected. "
                    "In a script, check that add succeeded and the returned ID exists before waiting. "
                    "Handle a failed wait explicitly; do not run the next step unconditionally after it. "
                    "Keep the task record until verification finishes. Pueue 4.0.4's wait implementation can return success when no matching tasks remain, "
                    "so an empty selection must not become evidence that work ran."
                }
            }
            section aria-labelledby="prepare" {
                h2 id="prepare" { "1. Prepare pinned binaries and a private queue" }
                p {
                    "Use a Linux machine with Python 3.11 or later and Bash. Obtain the client and daemon from the "
                    a href="https://github.com/Nukesor/pueue/releases/tag/v4.0.4" { "official 4.0.4 release" }
                    " or your trusted package source, and check both version outputs. The exercise rejects other versions instead of silently applying these observations to a different release. "
                    "It does not download software or require administrator privileges."
                }
                p {
                    "Download and inspect " a href="/assets/examples/pueue-success-v1/lab.py" download { "the Python success-boundary exercise" }
                    ". Supply absolute paths to the two executables. The script creates a temporary configuration, Unix socket and data directory, "
                    "then starts its own foreground daemon. Every client call selects that configuration explicitly. "
                    "It never targets your normal queue, changes your service manager or accepts a directory of your real files."
                }
                p {
                    "The fixture disables pause-on-failure behavior for its own daemon so failed dependencies can reach a terminal result. "
                    "It supplies a small environment rather than copying your credentials into synthetic task records. "
                    "The generated shell scripts only create sample files in the temporary directory. "
                    "On exit, the exercise shuts down its daemon and removes that generated directory; it does not run a reset against your existing tasks."
                }
            }
            section aria-labelledby="run" {
                h2 id="run" { "2. Run the Pueue task queue exercise" }
                pre { code { "python3 lab.py \\\n  --pueue /absolute/path/to/pueue \\\n  --pueued /absolute/path/to/pueued" } }
                p {
                    "Successful output is JSON containing seven check records and artifact_content_checked set to true. "
                    "The script bounds individual client calls with a timeout and fails if the daemon cannot start, "
                    "a task disappears, a status differs or a file assertion fails. A timeout or traceback is a failed exercise, not a slow pass. "
                    "The printed task outcomes describe only this disposable run."
                }
                p {
                    "The first task deliberately fails. Its dependent tries to create a marker but should never execute under normal scheduling. "
                    "Two independent tasks contrast an unguarded Bash pipeline with a straight-line script using set -e -o pipefail. "
                    "A final producer writes sample input; its dependent checks that input and writes a confirmation. "
                    "This positive control matters: checking only that a failed dependency did nothing would not establish that the queue could execute useful work."
                }
                p {
                    "If a result differs, keep the error locally, confirm both versions, and inspect the script before changing any assertion. "
                    "Do not turn a failing check into a pass by removing the success target or accepting a missing task. "
                    "The private fixture is removed after the run, so it is not a permanent log archive. "
                    "Its JSON is a compact record of the checks, not a dump of a user's commands or environment."
                }
            }
            section aria-labelledby="results" {
                h2 id="results" { "3. Read task outcomes alongside file evidence" }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Pueue outcome table, scroll horizontally" {
                    table class="task-table" {
                        caption { "Observed November 17, 2026; Pueue 4.0.4, Linux, six synthetic tasks" }
                        thead { tr { th scope="col" { "Task" } th scope="col" { "Stored result" } th scope="col" { "Success-wait exit" } th scope="col" { "File evidence" } } }
                        tbody {
                            tr { th scope="row" { "Explicit exit 7" } td { "Failed(7)" } td { "1" } td { "No output required" } }
                            tr { th scope="row" { "Depends on exit 7" } td { "DependencyFailed" } td { "1" } td { "Blocked marker absent" } }
                            tr { th scope="row" { "Unguarded false | cat" } td { "Success" } td { "0" } td { "Marker present" } }
                            tr { th scope="row" { "Strict Bash pipeline" } td { "Failed(1)" } td { "1" } td { "Strict marker absent" } }
                            tr { th scope="row" { "Sample producer" } td { "Success" } td { "0" } td { "Input file present" } }
                            tr { th scope="row" { "Successful dependent" } td { "Success" } td { "0" } td { "Confirmation content checked" } }
                        }
                    }
                }
                p {
                    "The seventh observation is the failed task's enqueue/completion pair: both clients returned 0. "
                    "That explains why a script built around add followed by ordinary wait can look green while the command failed. "
                    "The dependency row supplies a stronger negative control: the task record shows DependencyFailed and the attempted output is absent. "
                    "The final rows demonstrate the working path with an actual content check."
                }
                p {
                    "The pipeline example is a shell boundary, not a Pueue bug. "
                    "Bash normally uses the last pipeline command's exit status; cat can succeed after false fails. "
                    "The strict fixture uses Bash explicitly, enables pipefail and stops on this failed pipeline before writing its marker. "
                    "See the " a href="https://www.gnu.org/software/bash/manual/html_node/Pipelines.html" { "Bash pipeline rules" }
                    ". This particular example does not establish that set -e handles every shell construct. Test the script you intend to queue."
                }
            }
            section aria-labelledby="handoff" {
                h2 id="handoff" { "4. Give each queued task an acceptance contract" }
                p {
                    "Before queueing a real conversion, build or import, fill in the worksheet below. "
                    "The useful unit is one task and one observable output, not an entire terminal session. "
                    "For a report generator, the artifact check could verify required columns and a known sample row. "
                    "For an archive, it could list and restore a sample into another directory. Choose checks that would fail for the wrong result."
                }
                pre { code { "Command and working directory:\nCaptured task ID and record still present:\nRequired successful prerequisite IDs:\nExpected process result:\nExpected artifact path and content assertion:\nNegative control that proves the check can fail:\nObserved result and local evidence location:\nRetry decision after inspecting possible side effects:" } }
                p {
                    "Record paths and task IDs explicitly rather than assuming the newest task is yours. "
                    "Retain logs until you have compared the artifact with its requirement. "
                    "A command that exits zero after producing an empty or incorrect file can still be reported as successful. "
                    "Use a validator inside the command when appropriate, then independently inspect the destination before a consequential next step. "
                    "The worksheet is a proposed acceptance method, not a claim that Pueue validates your data for you."
                }
                p {
                    "For dependencies, let normal scheduling evaluate --after. "
                    "The " a href=(cli_source) { "4.0.4 CLI definitions" } " explicitly allow start with a task ID to bypass dependencies and parallel limits. "
                    "Do not use force-start as a dependency-safe recovery shortcut. "
                    "A paused group can also leave dependency failure unresolved while you repair a prerequisite. "
                    "Check the group state and the " a href="https://github.com/Nukesor/pueue/wiki/Get-started#dependencies" { "upstream dependency recovery guidance" }
                    " before reusing an existing chain; our running-queue fixture does not test that recovery path."
                }
            }
            section aria-labelledby="limits" {
                h2 id="limits" { "When Pueue is the wrong layer" }
                p {
                    "Pueue is worth evaluating when the work already makes sense as commands owned by one operator: "
                    "queue them, limit concurrency and inspect the results. "
                    "The " a href="https://github.com/Nukesor/pueue/blob/v4.0.4/README.md" { "project's own scope" }
                    " describes a single-user tool rather than a shared cluster scheduler. "
                    "Use a different selection process when you need multi-user authorization, distributed resource scheduling or application-level delivery guarantees. "
                    "This lab establishes none of those properties."
                }
                p {
                    "Application jobs are a different boundary. "
                    a href="https://shiprust.com/docs/background-jobs" { "ShipRust's background-job documentation" }
                    " gives a concrete example: Postgres job rows, transactional enqueueing and retrying workers. "
                    "That is a useful contrast when the requirement is to commit business data and its job together, rather than queue an operator's shell command. "
                    "The reference describes that implementation; it is not an endorsement or a comparative reliability test."
                }
                p {
                    "Even for local commands, inspect shell quoting and daemon lifecycle separately. "
                    "Pueue's " a href="https://github.com/Nukesor/pueue/wiki/Common-Pitfalls-and-Debugging" { "official pitfalls guide" }
                    " warns that a background ampersand can make a task finish before the detached work does. "
                    "A foreground daemon is also not proof of a service surviving terminal closure. "
                    "Our trial does not test logout, reboot, daemon crashes, remote connections, Windows or macOS. "
                    "Start with the disposable exercise, then validate the exact environment and output your workflow needs."
                }
            }
            section aria-labelledby="questions" {
                h2 id="questions" { "Common questions" }
                @for (q, a) in faqs { h3 { (q) } p { (a) } }
            }
            p class="guide-note" {
                "Next: inspect the " a href="/projects/pueue" { "Pueue source and project links" }
                ", run the private exercise, and write one acceptance contract before moving real work into a queue."
            }
            script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
        }
    }
}
