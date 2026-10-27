//! Source-linked CSV admission checks and a disposable, versioned exercise.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/qsv-csv-validation");
    let faqs = [
        (
            "Is a CSV valid if qsv can read it?",
            "Not necessarily. A structural check can accept negative quantities, unexpected status values and duplicate IDs. Validate a reviewed schema, then apply destination-specific rules before importing.",
        ),
        (
            "Does qsv validate preserve leading zeros?",
            "In this exercise, an order_id declared as a string passed a four-digit pattern and the source file retained 0007. Validation does not establish how your importer will interpret that value.",
        ),
        (
            "Does a .valid file mean the import succeeded?",
            "No. It contains rows accepted by the supplied schema after a validation failure. Our mixed fixture still exited 1. No destination import was performed.",
        ),
    ];
    let steps = [
        ("Check the installed build", "Record qsv --version and read qsv validate --help."),
        ("Run disposable failure cases", "Run the linked Python exercise and inspect all six cases."),
        ("Review an explicit schema", "Define identifier text, quantity bounds and allowed status values."),
        ("Inspect quarantine output", "Reconcile accepted and rejected records against the input."),
        (
            "Gate the real import separately",
            "Require both validation commands to succeed, then run destination-specific checks.",
        ),
    ];
    let schema = serde_json::json!({
        "@context": "https://schema.org", "@graph": [
            {"@type": "Article", "@id": url, "url": url, "headline": "qsv CSV validation: catch bad data before import",
             "datePublished": "2026-10-27T12:06:00Z", "dateModified": "2026-10-27T12:06:00Z",
             "author": {"@type": "Organization", "name": "Built with Rust editorial", "url": site_url}},
            {"@type": "HowTo", "name": "Validate CSV data with qsv before import", "step": steps.iter().enumerate().map(|(i,(name,text))| serde_json::json!({"@type": "HowToStep", "position": i+1, "name": name, "text": text})).collect::<Vec<_>>()},
            {"@type": "BreadcrumbList", "itemListElement": [
                {"@type": "ListItem", "position": 1, "name": "Projects", "item": site_url},
                {"@type": "ListItem", "position": 2, "name": "Guides", "item": format!("{site_url}/guides")},
                {"@type": "ListItem", "position": 3, "name": "qsv CSV validation", "item": url}]},
            {"@type": "FAQPage", "mainEntity": faqs.iter().map(|(q,a)| serde_json::json!({"@type":"Question", "name":q,"acceptedAnswer":{"@type":"Answer","text":a}})).collect::<Vec<_>>()}
        ]
    });
    html! {
        article class="python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / " a href="/guides" { "Guides" } " / qsv CSV validation" }
            header {
                h1 { "qsv CSV validation: catch bad data before import" }
                p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-10-27" { "October 27, 2026" } " · Sources and exercise checked October 27, 2026" }
            }
            p class="guide-lead" {
                "Use qsv CSV validation in two stages: check that the file is readable, then check its rows against an explicit JSON Schema. "
                "Neither stage proves an import will succeed. The six-case exercise below shows bad values that pass the first check, rows rejected by the second, and duplicate IDs that pass both."
            }
            p {
                "This is a practical trial of " a href="/projects/qsv" { "qsv, the Rust-powered command-line data toolkit" }
                ", for developers receiving CSV exports or building file-import pipelines. You need qsv and Python 3; the exercise uses only Python's standard library. "
                "It creates synthetic orders in a temporary folder, performs no real import and removes its fixtures when finished."
            }
            ol { @for (name, text) in &steps { li { strong { (name) ": " } (text) } } }
            nav aria-label="On this page" { ol {
                li { a href="#build" { "Check the installed build" } }
                li { a href="#exercise" { "Run six validation cases" } }
                li { a href="#schema" { "Write the import contract" } }
                li { a href="#quarantine" { "Inspect accepted and rejected rows" } }
                li { a href="#gate" { "Gate the import" } }
                li { a href="#limits" { "Choose the right boundary" } }
                li { a href="#questions" { "Common questions" } }
            } }
            section aria-labelledby="build" {
                h2 id="build" { "1. Check the qsv build before copying commands" }
                p {
                    "Install a binary for your operating system from the " a href="https://github.com/dathere/qsv/releases/tag/26.0.0" { "official qsv 26.0.0 release" }
                    ", or follow the project's installation instructions. Record the binary you actually run, not just a package-manager label. "
                    "The release includes several binary variants; check the available command rather than assuming every build has the same features. "
                    "The commands in this guide use the ordinary validate mode, without options that change its success convention."
                }
                pre tabindex="0" aria-label="Build and command checks, scroll horizontally" { code { "qsv --version\nqsv validate --help" } }
                p {
                    "We executed the downloadable exercise with the official qsv 26.0.0 x86_64 Linux musl binary and Python 3.11.2 on Linux on October 27, 2026. "
                    "It made twelve data-validation calls, plus a schema check. This is a functional observation for tiny inputs, not a speed comparison, a memory benchmark or evidence about every platform. "
                    "A different version may report different diagnostics. If an assertion fails, inspect the result before relying on that version in an import job."
                }
                p {
                    "Avoid describing the first command as a universal CSV certification. qsv's own " a href="https://github.com/dathere/qsv/blob/26.0.0/README.md#rfc-4180-csv-standard" { "CSV-standard notes" }
                    " explain its practical departures from strict RFC 4180. Here, the useful question is narrower: does this build reject our deliberately inconsistent record width? "
                    "It does. That does not tell us how every malformed quote or unusual dialect will be handled."
                }
            }
            section aria-labelledby="exercise" {
                h2 id="exercise" { "2. Run a six-case qsv CSV validation exercise" }
                p {
                    "Download " a href="/assets/examples/qsv-validation-v1/lab.py" download="lab.py" { "lab.py" }
                    " and read it before running. It resolves your qsv executable, removes inherited QSV_ settings, disables dotenv loading and runs in a new temporary working directory. "
                    "It writes only synthetic fixtures, their validation reports and mock import markers inside that directory. "
                    "It neither installs software nor contacts a database, spreadsheet service or import API."
                }
                pre tabindex="0" aria-label="Run the disposable validation exercise, scroll horizontally" { code { "python3 lab.py\n# Or select an installed binary explicitly:\npython3 lab.py --qsv /path/to/qsv" } }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Six observed validation outcomes, scroll horizontally" {
                    table class="task-table" {
                        caption { "Observed qsv 26.0.0 exits; 0 means success, 1 means these fixtures failed validation" }
                        thead { tr { th scope="col" { "Fixture" } th scope="col" { "Structural check" } th scope="col" { "Schema check" } th scope="col" { "Mock import" } } }
                        tbody {
                            tr { th scope="row" { "Two acceptable orders" } td { "0" } td { "0" } td { "Allowed" } }
                            tr { th scope="row" { "One good row, negative quantity, unknown status" } td { "0" } td { "1" } td { "Blocked" } }
                            tr { th scope="row" { "One row missing its status field" } td { "1" } td { "1" } td { "Blocked" } }
                            tr { th scope="row" { "Empty order identifier" } td { "0" } td { "1" } td { "Blocked" } }
                            tr { th scope="row" { "Unexpected note column" } td { "0" } td { "1" } td { "Blocked" } }
                            tr { th scope="row" { "Two identical order IDs and rows" } td { "0" } td { "0" } td { "Allowed: a deliberate limitation" } }
                        }
                    }
                }
                p {
                    "The mixed fixture is the important false green: the CSV is readable even though two orders violate our rules. "
                    "The ragged fixture tests a different failure, with two fields under a three-field header. "
                    "The empty-ID and extra-column cases check boundaries that a friendly example can miss. "
                    "Finally, the duplicate fixture makes an omitted rule visible. A green result only covers what the supplied schema actually expresses."
                }
                p {
                    "The exercise checks exit statuses, accepted/rejected row counts, sidecar-file behavior and unchanged source bytes. "
                    "It also asserts that the good file still contains the identifier text 0007 and 0042. "
                    "Its mock import is just a marker file: two green validation exits create it; any failure prevents it. "
                    "That makes the control flow observable without pretending we have tested your destination."
                }
            }
            section aria-labelledby="schema" {
                h2 id="schema" { "3. Write a CSV import contract, not just a sample profile" }
                p {
                    "Our synthetic order contract has three columns. An order ID is four digits of text, quantity is an integer of at least one, and status is either ready or hold. "
                    "The following is the complete schema embedded in the exercise. For a manual trial, save it as orders.schema.json next to a copy of an orders.csv file in a fresh scratch folder."
                }
                pre tabindex="0" aria-label="Complete orders JSON Schema, scroll horizontally" { code { r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "order_id": {"type": "string", "pattern": "^[0-9]{4}$"},
    "quantity": {"type": "integer", "minimum": 1},
    "status": {"type": "string", "enum": ["ready", "hold"]}
  },
  "required": ["order_id", "quantity", "status"],
  "additionalProperties": false
}"# } }
                p { "You can also download the identical " a href="/assets/examples/qsv-validation-v1/orders.schema.json" download="orders.schema.json" { "orders.schema.json" } " used in the exercise." }
                p {
                    "The " a href="https://json-schema.org/understanding-json-schema/reference/object" { "JSON Schema object reference" }
                    " distinguishes defining properties from requiring them and rejecting additional properties. "
                    "Those are separate decisions in this contract. Listing an order_id property alone is not the same as making that property required. "
                    "Likewise, a surprise note column should be allowed or rejected deliberately, rather than depending on what the importer happens to ignore."
                }
                p {
                    "Treat identifiers as identifiers. Declaring order_id as a string lets our 0007 value remain text during the schema check. "
                    "The " a href="https://json-schema.org/understanding-json-schema/reference/string#regular-expressions" { "JSON Schema pattern guidance" }
                    " explains why the anchors matter: a pattern can otherwise match only part of a value. "
                    "Our four-digit pattern is a rule for these synthetic IDs, not a recommendation for every order system. "
                    "The source file remaining unchanged also does not guarantee that a later spreadsheet or database loader preserves leading zeros. Test that separately."
                }
                p {
                    "An empty CSV cell is another trap. In our empty-ID run, the diagnostic says null is not of type string. "
                    "That is the behavior observed with this qsv version and schema, not a claim that CSV itself defines a null type. "
                    "Define your own missing-value policy before changing nullable types or trimming fields. Automatically accepting blanks would weaken this particular contract."
                }
                pre tabindex="0" aria-label="Validate the schema and CSV, scroll horizontally" { code { "qsv validate schema orders.schema.json\nqsv validate orders.csv\nqsv validate orders.csv orders.schema.json" } }
                p {
                    "You can use inferred schemas as a starting point, but review their assumptions. "
                    "The project's " a href="https://github.com/dathere/qsv/blob/26.0.0/docs/help/schema.md" { "schema-generation documentation" }
                    " warns about selecting representative training data and tuning inferred rules. "
                    "An allowed status missing from a sample is not necessarily an invalid status. Conversely, an observed negative quantity should not become acceptable merely because it appeared in the sample. "
                    "A contract needs an owner who knows what the destination is supposed to accept."
                }
            }
            section aria-labelledby="quarantine" {
                h2 id="quarantine" { "4. Inspect quarantine files without mistaking them for success" }
                p {
                    "In our mixed fixture, one record passed and two failed. qsv exited 1 and produced mixed.csv.valid, mixed.csv.invalid and mixed.csv.validation-errors.tsv. "
                    "The accepted file contained one data row; the rejected file contained two. "
                    "The error report identified quantity on data row 2 and status on data row 3. These row numbers are the observed report numbering, not a promise about physical lines in a file containing multiline cells."
                }
                p {
                    "A file named .valid is not a completed import. For an all-or-nothing batch, keep the entire delivery blocked until the owner resolves the rejected rows. "
                    "For a partial-acceptance workflow, first define who approves the split, how rejected rows are retried and how accepted rows avoid being imported twice. "
                    "Those policies are outside this exercise. Do not silently turn a rejected delivery into a successful job by importing whatever file is available."
                }
                p {
                    "The " a href="https://github.com/dathere/qsv/blob/26.0.0/docs/help/validate.md" { "version-pinned validate reference" }
                    " documents the default reports and exit convention. With our all-valid fixtures, no sidecar files were created. "
                    "Use a fresh working directory for each delivery so old .valid files cannot masquerade as current output. "
                    "The exercise creates a fresh directory on every invocation; a production job needs its own isolation and retention policy. Avoid options such as --valid-output unless you intentionally handle their different exit behavior."
                }
            }
            section aria-labelledby="gate" {
                h2 id="gate" { "5. Fail closed before calling the importer" }
                p {
                    "Separate validation from the action that consumes data. The basic shell shape below stops after either validation failure. "
                    "It deliberately contains no importer command: replace the final message only after adding your destination's checks and testing the failure path. "
                    "Run it against a stable staged copy of the delivery and its reviewed schema, not a file another process is still writing. Audit QSV_ environment settings and dotenv configuration too: unlike the isolated exercise, these manual commands inherit your shell settings."
                }
                pre tabindex="0" aria-label="Fail-closed validation gate, scroll horizontally" { code { "#!/bin/sh\nqsv validate orders.csv || exit 1\nqsv validate orders.csv orders.schema.json || exit 1\nprintf '%s\\n' 'Validation passed; destination checks still required.'" } }
                p {
                    "A successful validator should not authorize more than it checked. Our duplicate fixture passes both commands because this simple schema never requires cross-row uniqueness. "
                    "qsv has a custom uniqueCombinedWith keyword for explicit combination checks; the "
                    a href="https://github.com/dathere/qsv/blob/26.0.0/src/cmd/validate.rs" { "tagged validation source and help" }
                    " describe it. We did not exercise that feature here. If your destination requires a unique order ID, add and test the appropriate rule rather than assuming the default schema supplies it."
                }
                p {
                    "Keep database constraints, reference-data checks and import-transaction behavior in the acceptance plan too. "
                    "For example, an allowed status does not establish that an order refers to an existing customer. "
                    "A green CSV check also says nothing about whether the same batch was already applied. "
                    "Our suggested admission checklist is: freeze the input, validate the schema, check structure, validate rows, reconcile counts, enforce destination rules, then authorize the import. "
                    "Record the exact input and schema used so a later failure can be reproduced."
                }
            }
            section aria-labelledby="limits" {
                h2 id="limits" { "Where qsv fits, and where this trial stops" }
                p {
                    "Choose this qsv workflow when you want an explicit, scriptable admission check for a tabular file. "
                    "Use the " a href="/categories/data-and-ai" { "Data & AI tool directory" } " to compare that terminal workflow with a programmed DataFrame workflow. "
                    "If transformations and joins live in your application, explore " a href="/projects/polars" { "Polars" }
                    " and the " a href="/guides/rust-python-tools" { "guide to Rust-powered Python tools" }
                    ". This is a choice about where the work belongs, not a ranking of processing engines."
                }
                p {
                    "Before adopting qsv for a real feed, extend the trial with the dialect, encoding, quoted delimiters, multiline cells and empty-delivery behavior you actually receive. "
                    "Test the installed build's error handling under your scheduler, not just in an interactive shell. "
                    "This article does not test those inputs, large datasets, remote schemas, concurrent writers, spreadsheet formula handling or a real database load. "
                    "Its contribution is a small reproducible failure matrix and an admission boundary you can inspect, not a claim that your pipeline is now safe."
                }
                p { "Start with the " a href="/projects/qsv" { "qsv project listing and upstream links" } ", run the disposable exercise, then add one failure case from your own import contract before changing a working pipeline." }
            }
            section aria-labelledby="questions" { h2 id="questions" { "Common questions" } @for (question, answer) in &faqs { h3 { (question) } p { (answer) } } }
            script type="application/ld+json" { (PreEscaped(schema.to_string())) }
        }
    }
}
