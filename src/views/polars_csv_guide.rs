//! A pinned, synthetic CSV representation trial, not an import certification.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/polars-csv-schema-inference");
    let reference = "https://github.com/pola-rs/polars/blob/py-2.1.0/py-polars/src/polars/io/csv/functions.py";
    let faqs = [
        (
            "Does quoting a CSV identifier preserve leading zeros in Polars?",
            "Not in our Polars 2.1.0 numeric-only control: both 0007 and quoted 0007 became integer 7. Set that column to String when reading, then compare the exact expected identifiers.",
        ),
        (
            "Will infer_schema_length=None fix a Polars CSV import?",
            "It resolved the late text amount in our 101-row fixture, but the separate identifier column still became integers. Full inference is not an identifier or business-rule contract.",
        ),
        (
            "Is ignore_errors=True a safe way to keep every value?",
            "No. In our Polars 2.1.0 fixture, the late unknown amount became null while the row remained. Inspect the original text before deciding how to handle a failed conversion.",
        ),
    ];
    let steps = [
        ("Run the pinned exercise", "Use the downloadable synthetic lab with Python 3.11 and Polars 2.1.0."),
        ("Compare exact values", "Inspect identifier strings, the late amount, row counts and null counts."),
        ("Choose explicit types", "Read identifier columns as String; retain raw values for inspection."),
        ("Review rejected records", "Apply application rules separately before authorizing a real import."),
    ];
    let schema = serde_json::json!({
        "@context":"https://schema.org", "@graph":[
            {"@type":"Article","@id":url,"url":url,"headline":"Polars CSV schema: preserve IDs",
             "datePublished":"2026-11-01","dateModified":"2026-11-01",
             "author":{"@type":"Organization","name":"Built with Rust editorial","url":site_url}},
            {"@type":"HowTo","name":"Check Polars CSV inference and identifier preservation",
             "step":steps.iter().enumerate().map(|(i,(name,text))|serde_json::json!({"@type":"HowToStep","position":i+1,"name":name,"text":text})).collect::<Vec<_>>()},
            {"@type":"BreadcrumbList","itemListElement":[
                {"@type":"ListItem","position":1,"name":"Projects","item":site_url},
                {"@type":"ListItem","position":2,"name":"Guides","item":format!("{site_url}/guides")},
                {"@type":"ListItem","position":3,"name":"Polars CSV schema","item":url}]},
            {"@type":"FAQPage","mainEntity":faqs.iter().map(|(q,a)|serde_json::json!({"@type":"Question","name":q,"acceptedAnswer":{"@type":"Answer","text":a}})).collect::<Vec<_>>()}
        ]
    });
    html! {
        article class="python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / " a href="/guides" { "Guides" } " / Polars CSV schema" }
            header {
                h1 { "Polars CSV schema: preserve IDs" }
                p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-11-01" { "November 1, 2026" } " · Exercise and sources checked November 1, 2026" }
            }
            p class="guide-lead" {
                "Preserve leading-zero identifiers in Polars CSV reads by choosing String types before parsing, then checking exact values. "
                "A larger inference window can resolve a late mixed-type value without preserving identifier padding. "
                "This pinned, eight-case exercise separates those problems and checks rows and nulls before any real import is considered."
            }
            p {
                "Built with Rust ran the exercise with Python Polars 2.1.0 and Python 3.11.2 on Linux on November 1, 2026. "
                "Full inference kept a late unknown amount as text, but changed the first order identifier from 0001 to integer 1. "
                "Explicit String types retained every expected identifier. These are observations from small synthetic inputs, not a newly discovered bug or a performance comparison."
            }
            p { "Use this walkthrough when evaluating " a href="/projects/polars" { "Polars for a DataFrame workflow" }
                ". If you are still choosing application roles, start with our " a href="/guides/rust-python-tools" { "guide to Rust-powered Python tools" }
                ". This walkthrough focuses on what CSV inference preserved and what your program still needs to check." }
            ol { @for (name,text) in &steps { li { strong { (name) ": " } (text) } } }
            nav aria-label="On this page" { ol {
                li { a href="#run" { "Run the disposable exercise" } }
                li { a href="#outcomes" { "Read the eight outcomes" } }
                li { a href="#types" { "Choose a representation contract" } }
                li { a href="#review" { "Inspect rejected values" } }
                li { a href="#limits" { "Method and limits" } }
                li { a href="#questions" { "Common questions" } }
            } }
            figure class="rustic-restore-figure" {
                div class="hurl-flow" tabindex="0" aria-label="Identifier representation diagram, scroll horizontally on narrow screens" {
                    svg viewBox="0 0 680 150" role="img" aria-labelledby="polars-csv-flow-title polars-csv-flow-desc" {
                        title id="polars-csv-flow-title" { "The same CSV identifier can produce two representations" }
                        desc id="polars-csv-flow-desc" { "The source identifier 0007 becomes integer 7 under inference in our numeric-only control, but remains string 0007 with a String override. Check exact values after reading." }
                        rect x="5" y="30" width="190" height="90" rx="8" {}
                        text x="100" y="65" text-anchor="middle" { "CSV source: 0007" }
                        text class="flow-detail" x="100" y="92" text-anchor="middle" { "Identifier, not a quantity" }
                        rect x="285" y="5" width="385" height="60" rx="8" {}
                        text x="477" y="30" text-anchor="middle" { "Inferred integer: 7" }
                        text class="flow-detail" x="477" y="51" text-anchor="middle" { "Padding is no longer in the value" }
                        rect x="285" y="85" width="385" height="60" rx="8" {}
                        text x="477" y="110" text-anchor="middle" { "String override: 0007" }
                        text class="flow-detail" x="477" y="131" text-anchor="middle" { "Verify against the expected text" }
                        path d="M195 75h45V35h45m-7-6 7 6-7 6 M240 75v40h45m-7-6 7 6-7 6" {}
                    }
                }
                figcaption class="small muted" { "Built with Rust · November 1, 2026 · Polars 2.1.0 numeric-only control. Scroll sideways on mobile; the outcomes are also written below." }
            }
            section aria-labelledby="run" {
                h2 id="run" { "1. Run the disposable exercise" }
                p { "Download " a href="/assets/examples/polars-csv-v1/lab.py" download="lab.py" { "lab.py" }
                    " into a fresh scratch folder and read it. The script accepts no input paths, loads no customer files and performs no import. "
                    "It builds CSV bytes in memory, compares them with Python's standard-library CSV reader and raises an error if a checked outcome changes. "
                    "Its checks remain active even when Python runs with optimization enabled." }
                p { "The checked package is the " a href="https://github.com/pola-rs/polars/releases/tag/py-2.1.0" { "official Python Polars 2.1.0 release" }
                    ". The command below uses uv 0.13.6 syntax from the " a href="https://docs.astral.sh/uv/guides/scripts/#running-a-script-with-dependencies" { "official script-dependency guide" }
                    ". Install uv using its official instructions first. The same command works in a Windows terminal or a Unix shell; we executed the Linux path only. "
                    "uv may download Python and the pinned package. That setup uses the network; the lab itself makes no network requests." }
                pre tabindex="0" aria-label="Run the pinned synthetic exercise, scroll horizontally" { code { "uv run --no-project --python 3.11 --with polars==2.1.0 python lab.py" } }
                p { "The result is one JSON object containing eight cases and the original rejected record. A successful run exits normally. "
                    "The two expected parser failures are caught and reported as ComputeError; an unexpected result stops the script. "
                    "Do not substitute the latest package and label it the same experiment. The script deliberately refuses a different Polars version." }
                p { "No special Polars build, Rust compiler or database account is needed for this Python exercise. "
                    "Use a compatible Python environment and the official package for your platform. "
                    "If installation fails, resolve that environment issue rather than weakening a value-preservation check to get a green run." }
            }
            section aria-labelledby="outcomes" {
                h2 id="outcomes" { "2. Read the eight outcomes, not just the exit status" }
                p { "Two numeric-only controls contain order_id values 0007 and 0042 with amounts 12 and 34. "
                    "One control quotes both identifiers. The longer fixture has 101 data rows: identifiers 0001 through 0101, "
                    "amounts 1 through 100, then the text unknown in the final row. There are no empty fields or physical blank records in these inputs. "
                    "The " a href=(reference) { "2.1.0 API" } " defaults to a 100-row inference window, so the conflicting amount is just outside that sample." }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Eight observed Polars CSV outcomes, scroll horizontally" {
                    table class="task-table" {
                        caption { "Built with Rust's Polars 2.1.0 observations, November 1, 2026; tiny synthetic inputs, no real import" }
                        thead { tr { th scope="col" { "Read case" } th scope="col" { "Outcome" } th scope="col" { "Identifier / final amount" } } }
                        tbody {
                            tr { th scope="row" { "Numeric-only control" } td { "2 rows" } td { "Integer 7 / integer 34" } }
                            tr { th scope="row" { "Quoted-ID control" } td { "2 rows" } td { "Integer 7 / integer 34" } }
                            tr { th scope="row" { "Late value, defaults" } td { "ComputeError" } td { "No completed DataFrame" } }
                            tr { th scope="row" { "Late value, ID String only" } td { "ComputeError" } td { "Amount still fails" } }
                            tr { th scope="row" { "Full inference" } td { "101 rows; no nulls" } td { "Integer 1 / text unknown" } }
                            tr { th scope="row" { "Both columns String" } td { "101 rows; no nulls" } td { "Text 0001 / text unknown" } }
                            tr { th scope="row" { "Inference disabled" } td { "101 rows; no nulls" } td { "Text 0001 / text unknown" } }
                            tr { th scope="row" { "ignore_errors=True" } td { "101 rows; one amount null" } td { "Integer 1 / null" } }
                        }
                    }
                }
                p { "The quoted control is useful because quotation marks look like a request to keep text, but here they only served as CSV syntax. "
                    "The failed late-value reads prove a different point: no completed frame was available. They do not themselves demonstrate identifier conversion. "
                    "That behavior is established by the successful numeric controls and full-inference read." }
                p { "The last case is the misleading success. The row count remained 101, yet the original unknown token was no longer present in the amount column. "
                    "A row count alone would have missed it. The lab checks exact identifier lists, the final amount and per-column null counts, "
                    "so a successful read cannot silently stand in for all those conditions." }
            }
            section aria-labelledby="types" {
                h2 id="types" { "3. Choose types before identifiers become numbers" }
                p { "According to the " a href=(reference) { "Polars 2.1.0 CSV API source" }
                    ", inference samples an initial window, schema_overrides fixes selected types, and infer_schema=False makes unspecified columns strings. "
                    "For this fixture, the following are two alternative reads. Each snippet assumes late is the exact bytes generated by the downloadable lab; "
                    "neither is a complete standalone import program." }
                pre tabindex="0" aria-label="Two alternative String-preserving reads for Polars 2.1.0" { code { "import polars as pl\n\n# Choose the two known columns explicitly:\ntext = pl.read_csv(late, schema_overrides={\n    \"order_id\": pl.String,\n    \"amount\": pl.String,\n})\n\n# Or inspect all columns as strings:\ntext = pl.read_csv(late, infer_schema=False)" } }
                p { "Both reads preserved every expected decoded value in this fixture. An identifier override alone did not make the late amount readable; "
                    "the columns need separate decisions. Keeping amount as text makes the unexpected token inspectable, not acceptable as a numeric amount. "
                    "Decide what your application permits before converting it." }
                p { "Full inference is a diagnostic option, not the default recommendation for large files. It resolved this mixed amount column but still chose integers for the independent ID column. "
                    "The " a href="https://github.com/pola-rs/polars/blob/py-2.1.0/crates/polars-io/src/csv/read/schema_inference.rs" { "tagged inference implementation" }
                    " classifies observed values; it cannot know that your all-digit value is an order code. "
                    "Once 0007 has become integer 7, converting that integer to text cannot recover the original padding without another contract or source." }
            }
            section aria-labelledby="review" {
                h2 id="review" { "4. Inspect original values before deciding what to reject" }
                p { "The lab makes a separate, non-strict integer conversion while retaining the original text frame. "
                    "This Polars 2.1.0 snippet uses the tagged " a href="https://github.com/pola-rs/polars/blob/py-2.1.0/py-polars/src/polars/series/series.py" { "Series cast and null-check APIs" }
                    " and " a href="https://github.com/pola-rs/polars/blob/py-2.1.0/py-polars/src/polars/dataframe/frame.py" { "DataFrame filter API" }
                    ". Here text means the complete all-string frame from the previous section." }
                pre tabindex="0" aria-label="Retain original records while inspecting failed conversions" { code { "parsed = text[\"amount\"].cast(pl.Int64, strict=False)\nrejected = text.filter(parsed.is_null())\nprint(rejected.to_dicts())\n# This fixture: [{\"order_id\": \"0101\", \"amount\": \"unknown\"}]" } }
                p { "Because this fixture starts with no null amounts, the null mask identifies its one failed conversion. "
                    "On real input the same mask can include preexisting nulls. Define missing-value policy separately; do not call every null an invalid token. "
                    "Keep the original identifier and amount beside any parsed column so a reviewer can see what was actually supplied." }
                p { "An import contract also needs rules this exercise does not enforce: required fields, identifier width and uniqueness, "
                    "permitted ranges and destination relationships. A String column alone guarantees none of those. "
                    "For a separate shell-based schema trial, use the " a href="/guides/qsv-csv-validation" { "qsv CSV validation exercise" }
                    ". That is an optional companion, not a prerequisite for Polars or proof that your destination accepted anything." }
            }
            section aria-labelledby="limits" {
                h2 id="limits" { "Method, limits and the next useful check" }
                p { "The downloadable code is Built with Rust's original synthetic fixture. "
                    "Python's " a href="https://docs.python.org/3.11/library/csv.html" { "3.11 CSV reader reference" }
                    " supplies the independent string-oriented comparison for decoded input fields. "
                    "We also reproduced the central full-inference-versus-String result with a separately authored four-row fixture and a two-row inference window. "
                    "Neither check used customer records, analytics data or an external dataset." }
                p { "The eight published cases use eager read_csv with the normal CPU implementation, not PyArrow or GPU parsing. "
                    "The lab does not test malformed quoting, ragged records, arbitrary encodings, overflow, locale-specific numbers, dates, large-file memory use or blank-line handling. "
                    "String reads still have null rules: do not assume a bare empty field, quoted empty field and literal NULL mean the same thing. "
                    "Add explicit fixtures for your own conventions before depending on them." }
                p { "For lazy processing, the " a href=(reference) { "2.1.0 scan_csv reference in the tagged source" }
                    " describes a separate reader entry point. This exercise does not establish eager/lazy equivalence. "
                    "Constructing a lazy plan is not a completed read, and a projection that skips a troublesome column cannot establish that column's validity. "
                    "We measured no speed or memory advantage here." }
                p { "Next, copy the acceptance idea, not an unreviewed import policy: build a synthetic example of your identifier, "
                    "a late unexpected value and your missing-value convention. State the exact rows and values you expect before reading. "
                    "Then verify the complete output and review failures before connecting any destination. "
                    "Explore the " a href="/categories/data-and-ai" { "Data & AI projects" } " when choosing the surrounding workflow." }
            }
            section aria-labelledby="questions" {
                h2 id="questions" { "Common questions" }
                @for (question,answer) in &faqs { h3 { (question) } p { (answer) } }
            }
            script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
        }
    }
}
