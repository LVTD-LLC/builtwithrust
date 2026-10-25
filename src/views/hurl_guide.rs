//! A reproducible local HTTP contract exercise; no work on the request path.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/hurl-http-tests");
    let steps = [
        ("prepare", "Download the loopback fixture and check the prerequisites"),
        ("status", "Run a status-only check against a healthy response"),
        ("contract", "Add explicit header and JSON assertions"),
        ("break", "Deliberately break the response and inspect the failure"),
        ("adopt", "Move one verified contract into your own workflow"),
    ];
    let faqs = [
        (
            "Do Hurl assertions need --test?",
            "No. Hurl executes assertions with or without --test. Test mode changes output and provides a run summary; it also runs multiple input files in parallel unless you select --jobs 1.",
        ),
        (
            "Why can an HTTP 200 test pass the wrong JSON?",
            "A status assertion checks the response code, not the meaning of the body. Add assertions for the response fields and headers your caller actually requires, then deliberately break those values to verify the check fails.",
        ),
        (
            "Does this exercise prove my API is production-ready?",
            "No. It tests one synthetic response contract on loopback. It does not establish authentication, TLS, browser rendering, concurrency, resilience or security for a real service.",
        ),
    ];
    let schema = serde_json::json!({
        "@context": "https://schema.org", "@graph": [
            {"@type": "Article", "@id": url, "url": url, "headline": "Hurl HTTP tests: catch a false green",
             "datePublished": "2026-10-25", "dateModified": "2026-10-25",
             "author": {"@type": "Organization", "name": "Built with Rust", "url": site_url}},
            {"@type": "HowTo", "name": "Check an HTTP response contract with Hurl", "step": steps.iter().map(|(id,name)| serde_json::json!({
                "@type":"HowToStep", "name":name, "url":format!("{url}#{id}"), "text":name
            })).collect::<Vec<_>>()},
            {"@type": "BreadcrumbList", "itemListElement": [
                {"@type": "ListItem", "position": 1, "name": "Projects", "item": site_url},
                {"@type": "ListItem", "position": 2, "name": "Hurl HTTP tests", "item": url}
            ]},
            {"@type": "FAQPage", "mainEntity": faqs.iter().map(|(q,a)| serde_json::json!({
                "@type":"Question", "name":q, "acceptedAnswer":{"@type":"Answer", "text":a}
            })).collect::<Vec<_>>()}
        ]
    });
    html! {
        article class="hurl-guide python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / Hurl HTTP tests" }
            h1 { "Hurl HTTP tests: catch a false green" }
            p class="guide-lead" {
                "Use Hurl to send an HTTP request and assert the response code, headers and JSON values your caller needs. Start with a local fixture, confirm a healthy response passes, then deliberately break it. A green status-only check can hide a wrong payload; this walkthrough makes that failure visible before you adopt the check."
            }
            p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-10-25" { "October 25, 2026" } }
            p {
                a href="/projects/hurl" { "Hurl" } " is a Rust command-line tool for HTTP requests and response assertions. "
                "This exercise is for developers deciding whether plain-text API checks fit their workflow. You do not need a Rust compiler or a public API account. "
                "The original contribution is a small, reproducible response fixture with deliberately broken modes, not another list of HTTP clients."
            }
            ol class="hurl-outline" {
                @for (id, name) in &steps { li { a href=(format!("#{id}")) { (name) } } }
                li { a href="#limits" { "What was checked, and what was not" } }
                li { a href="#questions" { "Common questions" } }
            }
            p class="small muted python-table-hint" { "Scroll the diagram sideways, or focus it and use the arrow keys." }
            figure class="hurl-flow" tabindex="0" aria-label="HTTP contract exercise flow, scroll horizontally on small screens" {
                svg viewBox="0 0 680 125" role="img" aria-labelledby="hurl-flow-title hurl-flow-desc" {
                    title id="hurl-flow-title" { "A passing test needs a deliberate failure check" }
                    desc id="hurl-flow-desc" { "Run the healthy fixture, change one response property, then confirm the contract fails. A status-only check misses a wrong JSON value." }
                    @for (x, label, detail) in [(5, "Healthy response", "Check passes"), (235, "Change one property", "Keep the same test"), (465, "Broken response", "Check must fail")] {
                        rect x=(x) y="12" width="210" height="92" rx="8" {}
                        text x=(x+105) y="48" text-anchor="middle" { (label) }
                        text class="flow-detail" x=(x+105) y="77" text-anchor="middle" { (detail) }
                    }
                    path d="M215 58h20m-7-6 7 6-7 6 M445 58h20m-7-6 7 6-7 6" {}
                }
                figcaption class="small muted" { "Keep the test fixed while changing the response. Otherwise you have not checked whether the assertion catches the defect." }
            }
            section aria-labelledby="prepare" {
                h2 id="prepare" { "1. Prepare an isolated local fixture" }
                p {
                    "You need Hurl and Python 3, plus two terminal windows. Follow the "
                    a href="https://hurl.dev/docs/installation.html" { "official installation instructions" }
                    " for your operating system. The walkthrough was checked with Hurl 8.0.1 on Linux and Python 3.11.2 on October 25, 2026. "
                    "That is a tested combination, not a claim that every operating system or later release behaves identically. On Windows, use your Python launcher's equivalent if python3 is not available."
                }
                p {
                    "Download these three files into a new empty folder. Read the short server before running it. It serves a synthetic JSON response, binds only to 127.0.0.1, "
                    "and does not require credentials or your project's files. Nothing is uploaded to Built with Rust. The server exists only while its process is running."
                }
                ul class="hurl-downloads" {
                    li { a href="/assets/examples/hurl-http-tests-v1/server.py" download="server.py" { "server.py: local fixture" } }
                    li { a href="/assets/examples/hurl-http-tests-v1/status-only.hurl" download="status-only.hurl" { "status-only.hurl: response-code check" } }
                    li { a href="/assets/examples/hurl-http-tests-v1/contract.hurl" download="contract.hurl" { "contract.hurl: header and payload checks" } }
                }
                p {
                    "Run the server from that folder in terminal A. Leave it running while using terminal B for Hurl. "
                    "If port 8765 is already occupied, stop here and resolve the conflict rather than assuming the process answering there is this fixture. "
                    "These commands are for this page's versioned fixture; its Python HTTP-server API is documented in the "
                    a href="https://docs.python.org/3.11/library/http.server.html" { "Python 3.11 reference" } "."
                }
                pre tabindex="0" { code { "python3 server.py --mode healthy" } }
                p { "The process should report that it is listening on 127.0.0.1:8765. Stop it with Ctrl+C before changing modes. "
                    "Do not expose this demonstration server through a public interface or treat it as a production application." }
            }
            section aria-labelledby="status" {
                h2 id="status" { "2. Get a green status-only check" }
                p {
                    "The first Hurl file asks for the fixture's health response and requires HTTP 200. It deliberately says nothing about the payload. "
                    "With the healthy server running, execute the following in terminal B. Hurl's "
                    a href="https://hurl.dev/docs/running-tests.html" { "test-running reference" }
                    " and " a href="https://github.com/Orange-OpenSource/hurl/blob/8.0.1/docs/manual.md" { "manual" }
                    " describe these options; the commands and downloaded files were exercised with version 8.0.1."
                }
                pre tabindex="0" { code { "hurl --test --jobs 1 --connect-timeout 2 --max-time 3 status-only.hurl" } }
                p {
                    "Expect a successful file result and process exit code 0. The timeouts bound waiting for this tiny local example; they are not a latency target for a real service. "
                    "The single-job option runs at most one input file at a time; it does not establish dependencies between files or add checks to the response."
                }
                p { "Read the downloaded file before adding assertions. Its HTTP 200 line is already an assertion. "
                    "Hurl executes assertions without --test too; test mode supplies a summary and changes normal response output. "
                    "A file that succeeds has established only the properties it actually asserted." }
            }
            section aria-labelledby="contract" {
                h2 id="contract" { "3. Check the response contract" }
                p {
                    "A response contract is the small set of properties the caller relies on. Here the caller requires JSON, a status field with the string value ok, "
                    "and an items field with the number 2. The second file adds those requirements to the same request. "
                    "The syntax comes from Hurl's " a href="https://hurl.dev/docs/asserting-response.html" { "response-assertion reference" }
                    "; this is the exact downloadable file tested with Hurl 8.0.1."
                }
                pre tabindex="0" { code { (include_str!("../../assets/examples/hurl-http-tests-v1/contract.hurl")) } }
                p { "Run it while the server is still healthy. The same version-8.0.1 manual and test-running documentation apply to this invocation." }
                pre tabindex="0" { code { "hurl --test --jobs 1 --connect-timeout 2 --max-time 3 contract.hurl" } }
                p {
                    "Expect success again. The JSONPath queries select individual response fields; equality checks their values. Keep the number unquoted when the contract requires a number. "
                    "The content-type check accepts a JSON header with an additional charset parameter, but it is only a contains check, not a complete media-type parser. "
                    "These assertions are intentionally narrow: they do not describe every valid API response or replace a schema designed for your application."
                }
            }
            section aria-labelledby="break" {
                h2 id="break" { "4. Prove the test catches a bad response" }
                p {
                    "Stop terminal A with Ctrl+C. Restart the same fixture in wrong-json mode, then run both Hurl commands again without editing either test file. "
                    "This mode still answers HTTP 200 with JSON, but changes the status field. The command below is part of this page's versioned Python fixture."
                }
                pre tabindex="0" { code { "python3 server.py --mode wrong-json" } }
                p {
                    "The status-only file still succeeds. The contract file fails its JSON assertion. That difference is the false green: reachability looked healthy while a caller's expected value was wrong. "
                    "Inspect the reported assertion and the process exit status rather than treating any printed response as success. "
                    "Repeat with the other modes below, stopping the old process before each restart. Pass the mode name after --mode when starting server.py."
                }
                p class="small muted python-table-hint" { "Scroll the table sideways, or focus it and use the arrow keys." }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Fixture failure matrix, scroll horizontally on small screens" {
                    table class="task-table" {
                        caption { "Built with Rust loopback exercise: observed outcomes, October 25, 2026" }
                        thead { tr { th scope="col" { "Fixture mode" } th scope="col" { "Status-only file" } th scope="col" { "Contract file" } th scope="col" { "What it isolates" } } }
                        tbody {
                            tr { th scope="row" { "healthy" } td { "Pass" } td { "Pass" } td { "Expected synthetic response" } }
                            tr { th scope="row" { "wrong-json" } td { "Pass" } td { "Fail" } td { "Wrong value despite HTTP 200" } }
                            tr { th scope="row" { "wrong-header" } td { "Pass" } td { "Fail" } td { "Wrong content type despite parseable JSON" } }
                            tr { th scope="row" { "wrong-status" } td { "Fail" } td { "Fail" } td { "HTTP 503 instead of 200" } }
                            tr { th scope="row" { "malformed-json" } td { "Pass" } td { "Fail" } td { "HTTP 200 with an invalid JSON body" } }
                            tr { th scope="row" { "Server stopped" } td { "Fail" } td { "Fail" } td { "No connection; no assertion result" } }
                        }
                    }
                }
                p {
                    "A failed connection is not proof that the server returned a bad JSON value: no response was available to check. "
                    "Likewise, a failed header assertion identifies a different requirement from a failed status assertion. Keep these failure classes distinct when deciding what to investigate. "
                    "This table reports this fixture and these files, not an estimate of reliability across websites."
                }
            }
            section aria-labelledby="adopt" {
                h2 id="adopt" { "5. Move one useful check into your workflow" }
                p {
                    "Choose one endpoint you control and one caller expectation worth protecting. Work on a disposable branch or staging setup. "
                    "Change the URL and assertions to match that contract, including its expected status, rather than copying this health response into an unrelated API. "
                    "First verify a known-good response, then make a reversible change that should violate the contract. If the check stays green, improve the assertion before relying on it in CI."
                }
                p {
                    "Keep the first trial independent of unrelated changes. The " a href="/guides/rust-terminal-tools" { "terminal-workflow guide" }
                    " separates terminal, shell and history choices; none is required to adopt Hurl. Browse "
                    a href="/categories/developer-tools" { "other Rust developer tools" }
                    " if the problem is linting, bundling or editing instead of checking HTTP responses. Hurl is an HTTP client here, not your application's implementation language."
                }
                p {
                    "Preserve TLS certificate verification when moving beyond loopback; this plain-HTTP fixture does not exercise TLS or authentication. "
                    "Avoid putting live tokens or private response bodies into committed example files, logs or shared reports. "
                    "Test mode is not a privacy boundary: debug output, reporting options and other output settings can still disclose response content. "
                    "Review what your chosen CI command stores, and keep credentials in the environment or your established secret mechanism."
                }
                p {
                    "Once the good response passes and a deliberate defect fails, keep the check small enough that its purpose is obvious in review. "
                    "A successful smoke check does not establish browser behavior, load capacity or security. Those need their own tests. "
                    "Stop the local server with Ctrl+C when finished; the downloaded folder can be removed when you no longer need the exercise. "
                    "Visit the " a href="/projects/hurl" { "Hurl project listing" } " for its website and source links, or "
                    a href="/submit" { "submit a Rust-powered tool" } " that solves a different workflow."
                }
            }
            section aria-labelledby="limits" {
                h2 id="limits" { "What was checked, and what was not" }
                p {
                    "Built with Rust ran the two downloadable files against each named mode of its own Python loopback fixture on Linux, using Hurl 8.0.1 and Python 3.11.2. "
                    "The server was restarted between modes, and connection refusal was checked after stopping it. The files shown above are the same embedded assets available for download. "
                    "No private account data, production API or remote credentials were used. These are reproducible functional examples, not a performance benchmark, cross-platform certification or security audit."
                }
                p {
                    "If your local result differs, check the installed Hurl version, which process owns the port, the selected mode and the exact downloaded file before changing your application. "
                    "Use the " a href="https://github.com/Orange-OpenSource/hurl/releases/tag/8.0.1" { "Hurl 8.0.1 release" }
                    " for the tested release and the current official documentation for later syntax. A future version or a different network environment deserves a fresh check."
                }
            }
            section aria-labelledby="questions" {
                h2 id="questions" { "Common questions" }
                @for (q, a) in &faqs { h3 { (q) } p { (a) } }
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string())) }
    }
}
