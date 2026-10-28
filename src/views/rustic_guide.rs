//! Reproducible synthetic restore acceptance exercise, rendered only on cache misses.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/rustic-backup-restore");
    let steps = [
        ("prepare", "Prepare the pinned tool and disposable exercise"),
        ("run", "Restore known files into empty destinations"),
        ("verify", "Test the existing-file verification boundary"),
        ("extras", "Check extra files and expected failures"),
        ("adopt", "Choose acceptance checks for your own backup workflow"),
    ];
    let faqs = [
        (
            "Does a successful rustic restore prove every destination file is correct?",
            "No. In rustic 0.11.4, default restore can trust existing file size and modification time. Use --verify-existing when existing contents need verification, and independently compare expected file paths and content hashes. Extra destination files are retained by default.",
        ),
        (
            "Is rustic check --read-data the same as a restore test?",
            "No. check --read-data reads and checks repository pack data. A restore exercise checks whether selected snapshot files can be recovered into a destination. Neither alone proves that an application can use those files or that metadata and remote recovery requirements are satisfied.",
        ),
        (
            "Can I run this exercise against my current backups?",
            "No. The downloadable exercise accepts only a rustic binary path. It generates its own synthetic source files, local repository, password and destinations. It is not a migration script, retention policy or instruction to modify an existing backup repository.",
        ),
    ];
    let schema = serde_json::json!({
        "@context":"https://schema.org", "@graph":[
            {"@type":"Article", "@id":url, "url":url, "headline":"Rustic backup restore: verify the files",
             "datePublished":"2026-10-28", "dateModified":"2026-10-28",
             "author":{"@type":"Organization", "name":"Built with Rust", "url":site_url}},
            {"@type":"HowTo", "name":"Run a disposable rustic restore acceptance exercise",
             "step":steps.iter().map(|(id,name)|serde_json::json!({"@type":"HowToStep", "name":name, "text":name, "url":format!("{url}#{id}")})).collect::<Vec<_>>()},
            {"@type":"BreadcrumbList", "itemListElement":[
                {"@type":"ListItem", "position":1, "name":"Projects", "item":site_url},
                {"@type":"ListItem", "position":2, "name":"Guides", "item":format!("{site_url}/guides")},
                {"@type":"ListItem", "position":3, "name":"Rustic backup restore", "item":url}]},
            {"@type":"FAQPage", "mainEntity":faqs.iter().map(|(q,a)|serde_json::json!({"@type":"Question", "name":q, "acceptedAnswer":{"@type":"Answer", "text":a}})).collect::<Vec<_>>()}
        ]
    });
    html! {
        article class="hurl-guide python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / " a href="/guides" { "Guides" } " / Rustic backup restore" }
            h1 { "Rustic backup restore: verify the files" }
            p class="guide-lead" {
                "Test a Rustic backup restore in a disposable repository before trusting your recovery workflow. Restore a selected snapshot into an empty directory, compare file paths and hashes, then deliberately change an existing destination file. A successful restore exit code is not the same as verified contents or an exact destination mirror."
            }
            p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-10-28" { "October 28, 2026" } }
            p {
                "Built with Rust's controlled Linux exercise found that rustic 0.11.4 returned success after a destination file's bytes were changed but its size and modification time were preserved. "
                "Repeating the restore with " code { "--verify-existing" } " recovered the expected bytes. That is documented behavior, not evidence of a broken repository. "
                "The downloadable exercise makes the distinction visible without touching your files or real backups."
            }
            p { a href="/projects/rustic" { "Rustic" } " is a Rust backup CLI. This walkthrough is for developers and self-hosters evaluating its recovery workflow, not a general backup-product ranking. "
                "For other application roles, explore the " a href="/categories/infrastructure" { "Infrastructure directory" } "." }
            ol class="hurl-outline" { @for (id,name) in &steps { li { a href=(format!("#{id}")) { (name) } } } li { a href="#limits" { "Method and limits" } } li { a href="#questions" { "Common questions" } } }
            p class="small muted python-table-hint" { "Scroll the diagram sideways, or focus it and use the arrow keys." }
            figure class="rustic-restore-figure" {
                div class="hurl-flow" tabindex="0" aria-label="Restore verification workflow, horizontally scrollable" {
                svg viewBox="0 0 680 125" role="img" aria-labelledby="rustic-flow-title rustic-flow-desc" {
                    title id="rustic-flow-title" { "A restore exit code is only one acceptance check" }
                    desc id="rustic-flow-desc" { "Select the intended snapshot, restore into an empty destination, then compare expected paths and file hashes. Repository checks and application checks answer different questions." }
                    @for (x,label,detail) in [(5,"Select snapshot","Know which version"),(235,"Restore separately","Keep source untouched"),(465,"Compare contents","Paths plus hashes")] {
                        rect x=(x) y="12" width="210" height="92" rx="8" {}
                        text x=(x+105) y="48" text-anchor="middle" { (label) }
                        text class="flow-detail" x=(x+105) y="77" text-anchor="middle" { (detail) }
                    }
                    path d="M215 58h20m-7-6 7 6-7 6 M445 58h20m-7-6 7 6-7 6" {}
                }
                }
                figcaption class="small muted" { "Built with Rust · October 28, 2026. Separate command completion, file recovery and application recovery; none is a substitute for the others." }
            }
            section aria-labelledby="prepare" {
                h2 id="prepare" { "1. Prepare a pinned, disposable exercise" }
                p { "Use Python 3.11 and rustic 0.11.4 for the checked combination. Get the tool from the "
                    a href="https://github.com/rustic-rs/rustic/releases/tag/v0.11.4" { "official v0.11.4 release" }
                    " and follow its installation and artifact-verification instructions for your platform. Built with Rust checked the x86_64 Linux musl archive against its published SHA-256 file; that checksum comparison was not an independent signature verification. "
                    "No Rust compiler or cloud-storage account is needed for this local exercise." }
                p { "Download " a href="/assets/examples/rustic-restore-v1/restore-check.py" download="restore-check.py" { "restore-check.py" }
                    " into a working folder and read it before running it. The only argument is an installed binary path. The script deliberately has no option for your source directory, password or repository. "
                    "It creates a fresh private temporary directory with synthetic text, an empty file and binary content. Its generated repository, cache and restore destinations live there." }
                p { "Each rustic child process receives a whitelisted environment, an explicit empty profile and local repository/cache paths. Inherited rustic settings and profile hooks are not used. "
                    "The fixture invokes no shell, remote backend, pruning or deletion command. You still need to trust the binary you install and inspect the downloaded script; an isolated exercise is not a sandbox for an untrusted executable." }
            }
            section aria-labelledby="run" {
                h2 id="run" { "2. Restore into empty destinations and compare" }
                p { "From the folder holding the download, run the command below, replacing the binary path. The wrapper uses Python's "
                    a href="https://docs.python.org/3.11/library/subprocess.html" { "subprocess API" } " and "
                    a href="https://docs.python.org/3.11/library/tempfile.html" { "temporary-directory API" }
                    ". Rustic commands are pinned to its " a href="https://github.com/rustic-rs/rustic/tree/v0.11.4/src/commands" { "v0.11.4 command source and help" }
                    "; the script refuses a different reported rustic version. This is the command for this page's versioned fixture, not an instruction to restore your own repository." }
                pre tabindex="0" { code { "python3 restore-check.py /absolute/path/to/rustic" } }
                p { "The script prints its temporary workspace, initializes a local repository and backs up a source tree under the snapshot path " code { "fixture" }
                    ". It saves the returned snapshot ID, restores that subtree into a separate empty destination, then compares relative filenames and SHA-256 content hashes. "
                    "It changes a source file, adds another file, takes a second snapshot and checks both the revised restore and the earlier snapshot. The earlier result must still match the earlier source manifest, not today's source." }
                p { "A snapshot is a saved version, not simply the current contents of your working directory. The "
                    a href="https://rustic.cli.rs/docs/commands/restore/intro.html" { "official restore documentation" }
                    " explains snapshot selection and subtree paths. This fixture uses explicit returned IDs so another snapshot cannot silently change what its checks mean. "
                    "For an existing repository, inspect the snapshot's host, paths and time before choosing a version; do not copy this exercise's generated IDs into another context." }
            }
            section aria-labelledby="verify" {
                h2 id="verify" { "3. Test the existing-file boundary" }
                p { "An empty destination is the simplest comparison because there are no existing files to reuse. A reused destination asks a different question. "
                    a href="https://github.com/rustic-rs/rustic/releases/tag/v0.11.4" { "Rustic 0.11.4" }
                    " exposes " code { "rustic restore --help" } ", checked from the released binary, which describes " code { "--verify-existing" } " as reading existing files rather than trusting matching modification time and size. The option is not a promise to verify every aspect of your recovery plan." }
                p { "The fixture deliberately overwrites the restored text file with different bytes of exactly the same length, then restores its previous modification timestamp. "
                    "A default restore returns success but leaves those bytes different from the snapshot. The script expects that mismatch: it is the negative control showing why a green command alone is insufficient. "
                    "It then repeats the restore with " code { "--verify-existing" } " and requires the full expected content manifest to match again." }
                p { "Do not interpret this as a corruption-rate estimate or a reason to distrust every restore. It is a controlled edge case for a documented optimization. "
                    "The practical decision is whether your acceptance process checks the recovered bytes it actually depends on. A backup command that finished and a restore command that finished are useful signals, but neither replaces that check." }
            }
            section aria-labelledby="extras" {
                h2 id="extras" { "4. Separate extra files from failed recovery" }
                p { "The script adds a destination-only file and runs default restore again. That extra file remains. Default restore is not an exact-mirror operation; the same version's help documents a separate "
                    code { "--delete" } " option for removing destination files absent from the snapshot. This exercise never invokes it. "
                    "For a first recovery drill, use a new empty directory rather than solving a comparison mismatch by deleting things from a destination you care about." }
                p { "Next, the wrapper tries a generated wrong password and a repository path that does not exist. Both calls must fail, and neither destination may appear. "
                    "These are local failure checks, not simulations of a lost cloud account or a damaged drive. If the wrapper times out, reports a mismatch or exits nonzero, the exercise did not complete. "
                    "Do not convert missing results into a pass. Keep the printed temporary workspace for inspection and compare the installed tool version with the fixture's prerequisite." }
                p { "The result file records command exits plus named content checks. The expected summary includes " code { "default_same_metadata_bytes_match: false" }
                    ", " code { "verify_existing_bytes_match: true" } " and " code { "additional_file_retained: true" }
                    ". The false value is intentional. It describes the deliberate changed-file case, not a failure of the clean restores." }
            }
            section aria-labelledby="adopt" {
                h2 id="adopt" { "5. Turn the exercise into an acceptance decision" }
                p { "Use the exercise to choose checks, not to adopt a complete backup policy. Write down the snapshot-selection rule, the expected files, the independent content comparison and the next application-level check. "
                    "For a static document, opening the recovered file may be a relevant extra check. For a database, file hashes alone do not establish a consistent, usable database restore; follow that application's backup and recovery procedure." }
                p { "Repository validation is another layer. The script also runs " code { "check --read-data" } ". The "
                    a href="https://rustic.cli.rs/docs/commands/misc/check.html" { "official integrity-check documentation" }
                    " distinguishes a structural repository check from reading pack data. That repository-side check does not compare a reused destination's current files or prove that an application starts correctly. Keep those results separate." }
                p { "Do not point this wrapper at an existing backup: it has no supported option to do so. Plan a separate recovery drill after reviewing official configuration, storage, retention and recovery requirements. "
                    "The " a href="/projects/rustic" { "Rustic project page" } " links to the upstream tool and its mixed-client safety caveat. "
                    "If you use restic too, read the " a href="https://rustic.cli.rs/docs/FAQ.html#can-i-use-rustic-with-my-existing-restic-repositories" { "official compatibility FAQ" }
                    ": a shared repository format does not make overlapping restic pruning and rustic writes safe. No mixed-client concurrency was tested here." }
                p { "When finished, inspect results.json and remove only the exact temporary workspace printed by the fixture using your normal file-management tools. "
                    "It contains generated passwords and synthetic data, not credentials for your backups. The script intentionally leaves it intact and never performs automatic cleanup. "
                    "For another example of testing a deliberately wrong success signal, see the " a href="/guides/hurl-http-tests" { "Hurl response-contract exercise" } "." }
            }
            section aria-labelledby="limits" {
                h2 id="limits" { "Method and limits" }
                p { "Built with Rust executed the downloadable wrapper with rustic 0.11.4 and Python 3.11.2 on x86_64 Linux on October 28, 2026. "
                    "The expected manifests were computed from synthetic source files before each backup; restored relative paths and content hashes were compared with those saved expectations. "
                    "The changed-file case preserved size and modification time deliberately. No visitor analytics, customer files, production repositories or private backup data contributed to these observations." }
                p { "These are functional examples, not a survey, benchmark, security review or durability guarantee. The manifest does not validate ownership, permissions, timestamps, extended attributes, symlinks or hard links; the fixture disables ownership restoration. "
                    "Remote backends, Windows, macOS, filesystem-specific edge cases, disaster recovery, application consistency and concurrent clients were not checked. "
                    "If another version or filesystem changes the expected behavior, stop and investigate rather than editing the checks until they turn green." }
            }
            section aria-labelledby="questions" { h2 id="questions" { "Common questions" } @for (q,a) in &faqs { details { summary { (q) } p { (a) } } } }
            script type="application/ld+json" { (PreEscaped(schema.to_string())) }
        }
    }
}
