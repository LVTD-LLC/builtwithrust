//! Native, cached editorial comparison with a pinned disposable exercise.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let url = format!("{site_url}/guides/dufs-vs-miniserve");
    let dufs = "https://github.com/sigoden/dufs/blob/v0.47.0/README.md#access-control";
    let mini = "https://github.com/svenstaro/miniserve/blob/v0.35.0/README.md#usage";
    let faqs = [
        (
            "Does miniserve support writable WebDAV?",
            "The miniserve 0.35.0 documentation describes read-only WebDAV. In our disposable trial, PROPFIND returned 207 while PUT returned 404, including with browser uploads enabled. Choose Dufs for evaluation when a DAV client must write; then test that client and the intended account rules.",
        ),
        (
            "Why can a Dufs read-write account fail to upload?",
            "Dufs account permissions are limited by globally enabled operations. In our Dufs 0.47.0 trial, an account with a read-write rule received 403 for PUT until global upload was enabled. After enabling uploads, path and account rules still determined which writes succeeded.",
        ),
        (
            "Can two miniserve accounts have different upload permissions?",
            "Our miniserve 0.35.0 profile accepted two accounts and enabled uploads to inbox. Both accounts could upload there, even though one was named reader. The names did not create roles. For per-account writable paths, evaluate Dufs rules rather than treating miniserve account names as permissions.",
        ),
    ];
    let schema = serde_json::json!({
        "@context":"https://schema.org", "@graph":[
            {"@type":"Article","@id":url,"url":url,"headline":"Dufs vs miniserve: who can write?",
             "datePublished":"2026-11-03","dateModified":"2026-11-03",
             "author":{"@type":"Organization","name":"Built with Rust editorial","url":site_url}},
            {"@type":"BreadcrumbList","itemListElement":[
                {"@type":"ListItem","position":1,"name":"Projects","item":site_url},
                {"@type":"ListItem","position":2,"name":"Guides","item":format!("{site_url}/guides")},
                {"@type":"ListItem","position":3,"name":"Dufs vs miniserve","item":url}]},
            {"@type":"FAQPage","mainEntity":faqs.iter().map(|(q,a)|serde_json::json!({
                "@type":"Question","name":q,"acceptedAnswer":{"@type":"Answer","text":a}
            })).collect::<Vec<_>>()}
        ]
    });
    html! {
        article class="python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / " a href="/guides" { "Guides" } " / Dufs vs miniserve" }
            header {
                h1 { "Dufs vs miniserve: who can write?" }
                p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-11-03" { "November 3, 2026" }
                    " · Sources and exercise checked November 3, 2026" }
            }
            p class="guide-lead" {
                "Choose miniserve for a temporary browser share where accepted accounts use the same enabled operations. "
                "Evaluate Dufs when different accounts need different writable paths or a WebDAV client must write. "
                "This Dufs vs miniserve comparison uses a disposable, version-pinned exercise to show why an account name, an upload button and a successful directory listing are different kinds of evidence."
            }
            p { "Dufs and miniserve are Rust file-sharing servers you run against a local directory. Both can offer browser downloads and optional uploads; their permission models and WebDAV write support differ." }
            p {
                "Built with Rust ran four local server profiles with Dufs 0.47.0, miniserve 0.35.0 and Python 3.11.2 on Linux. "
                "Fourteen checks paired HTTP responses with file contents or file absence. "
                "Both tools served the sample. The useful difference was which identity could change which path through which interface, not a speed ranking."
            }
            p { "Start with the " a href="/projects/dufs" { "Dufs catalog entry" } " or " a href="/projects/miniserve" { "miniserve catalog entry" }
                " for project links. For other jobs, use the " a href="/categories/infrastructure#file-sharing-workflows" { "infrastructure tool chooser" }
                ". Neither tool should be assumed to provide synchronization, backup retention or a managed collaboration service merely because it can serve files." }
            nav aria-label="On this page" { ol {
                li { a href="#choose" { "Choose by sharing policy" } }
                li { a href="#permissions" { "Separate identities, paths and operations" } }
                li { a href="#run" { "Run the four-profile exercise" } }
                li { a href="#outcomes" { "Read the observed outcomes" } }
                li { a href="#worksheet" { "Write your acceptance contract" } }
                li { a href="#limits" { "Check the boundaries we did not test" } }
                li { a href="#questions" { "Common questions" } }
            } }
            section aria-labelledby="choose" {
                h2 id="choose" { "Dufs vs miniserve: choose by sharing policy" }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="File server decision table, scroll horizontally" {
                    table class="task-table" {
                        caption { "Selection guidance from pinned documentation and our local exercise; not a security rating" }
                        thead { tr { th scope="col" { "Your actual requirement" } th scope="col" { "Start here" } th scope="col" { "Acceptance question" } } }
                        tbody {
                            tr { th scope="row" { "A trusted group downloads one folder" } td { "miniserve; Dufs also fits" } td { "Can the intended account read the exact sample?" } }
                            tr { th scope="row" { "One writable subtree per account" } td { "Dufs path rules" } td { "Does an allowed write succeed while a sibling-path write fails?" } }
                            tr { th scope="row" { "Browser uploads for all accepted accounts" } td { "miniserve upload directory" } td { "Is shared upload permission what you intended?" } }
                            tr { th scope="row" { "A WebDAV client creates files" } td { "Dufs with required operations enabled" } td { "Can the actual client write and read back, not merely list?" } }
                            tr { th scope="row" { "Uploaders cannot read submitted files" } td { "Neither example here establishes that" } td { "Does direct GET fail for every identity that must not read?" } }
                        }
                    }
                }
                p {
                    "miniserve earns its place when the sharing policy is small: serve this folder, accept these credentials, optionally accept uploads here. "
                    "Do not add account-specific policy machinery when everyone legitimately has the same access. "
                    "Its " a href=(mini) { "0.35.0 options" } " also include opt-in deletion and directory creation; calling it a download-only tool would be misleading. "
                    "Those options are separate from writable WebDAV."
                }
                p {
                    "Dufs is the more direct candidate when the policy names different accounts and paths. "
                    "Its " a href=(dufs) { "0.47.0 access-control documentation" } " assigns read-only or read-write rules to path scopes. "
                    "That is useful for a reader of the whole share and a writer limited to an inbox. "
                    "It also creates more policy to inspect: a plausible-looking rule is not proof the finished setup matches your intention."
                }
            }
            section aria-labelledby="permissions" {
                h2 id="permissions" { "Dufs vs miniserve permissions: three separate decisions" }
                p {
                    "Authentication answers which credentials the server accepts. A path rule answers where an identity may act. "
                    "An operation switch answers whether the server enables an action at all. "
                    "Write these as separate requirements before starting either tool; do not infer permissions from a username such as reader."
                }
                p {
                    "In Dufs, our writer had a read-write account rule but could not upload until the global upload operation was enabled. "
                    "After that, the writer could create a file inside inbox, the read-only account could read its exact bytes, "
                    "and that read-only account still could not upload. The writer could not create a sibling file outside its allowed subtree. "
                    "Deletion stayed disabled despite the writer's read-write rule. These are deliberately different checks."
                }
                p {
                    "In miniserve, our upload profile accepted accounts named writer and reader. Both uploaded successfully into the enabled directory. "
                    "That result agrees with the " a href="https://github.com/svenstaro/miniserve/blob/v0.35.0/src/auth.rs" { "pinned authentication implementation" }
                    ": it accepts matching credentials; the names do not assign different upload roles. "
                    "A reverse proxy could introduce another policy layer, but that would be a different setup requiring its own checks."
                }
                p {
                    "WebDAV is a further interface decision. PROPFIND asks for resource properties or a directory listing; PUT sends file contents. "
                    "Our miniserve DAV profile answered PROPFIND but did not accept PUT. Enabling its browser-upload endpoint did not change that result. "
                    "The " a href="https://github.com/svenstaro/miniserve/blob/v0.35.0/src/main.rs" { "0.35.0 DAV handler" }
                    " uses a read-only method set. Do not mistake a mounted directory or a browser-upload success for a writable DAV connection."
                }
            }
            section aria-labelledby="run" {
                h2 id="run" { "Run the disposable four-profile exercise" }
                p {
                    "Download " a href="/assets/examples/file-server-v1/lab.py" download="lab.py" { "lab.py" }
                    " and read it first. It needs Python 3.11 or newer and two executable paths, not your existing share directory. "
                    "Obtain the checked binaries from the official " a href="https://github.com/sigoden/dufs/releases/tag/v0.47.0" { "Dufs 0.47.0 release" }
                    " and " a href="https://github.com/svenstaro/miniserve/releases/tag/v0.35.0" { "miniserve 0.35.0 release" }
                    ". Use trusted packages for your platform and inspect the projects' installation instructions. The lab itself downloads nothing."
                }
                pre tabindex="0" aria-label="Run the local file-server exercise, scroll horizontally" { code {
                    "python3 lab.py --dufs /absolute/path/to/dufs --miniserve /absolute/path/to/miniserve"
                } }
                p {
                    "The script rejects other version strings, generates fresh throwaway credentials and creates a new temporary share for each profile. "
                    "Each child receives a small environment without inherited DUFS or MINISERVE settings. "
                    "It binds to 127.0.0.1, ignores HTTP proxy settings and preserves the immediate response code instead of following redirects. "
                    "Only the supplied local executables run; their version checks also execute code, so the paths must point to binaries you trust."
                }
                p {
                    "Each profile starts a server, sends bounded requests, checks its synthetic files and stops the process. "
                    "The temporary files are removed when the run ends. A failed startup or changed observation stops the exercise; "
                    "it does not weaken an expected denial to get a passing result. Python optimization does not remove these checks. "
                    "A small free-port race can make startup fail, in which case inspect the environment and rerun the disposable trial."
                }
                p {
                    "Success prints JSON with fourteen checks, version strings and SHA-256 hashes of your binaries. "
                    "Compare package provenance as well as version output; a matching label alone is not a supply-chain guarantee. "
                    "The tested path was Linux with official x86_64 musl release binaries. Other operating systems, browsers and DAV clients remain untested here."
                }
            }
            section aria-labelledby="outcomes" {
                h2 id="outcomes" { "What our fourteen checks observed" }
                p {
                    "The four profiles separate the permission questions. The table groups their fourteen assertions. "
                    "A positive write checks exact bytes on disk. A denied creation checks that the target does not exist; "
                    "the denied deletion checks that the original bytes remain. Three successful GET checks also compare response bodies."
                }
                div class="task-table-wrap" tabindex="0" role="region" aria-label="Observed file-server outcomes, scroll horizontally" {
                    table class="task-table" {
                        caption { "Built with Rust · November 3, 2026 · Dufs 0.47.0 / miniserve 0.35.0 · Linux loopback HTTP" }
                        thead { tr { th scope="col" { "Profile and request" } th scope="col" { "Immediate HTTP result" } th scope="col" { "File-state evidence" } } }
                        tbody {
                            tr { th scope="row" { "Dufs: rw account, no global upload" } td { "GET 200; PUT 403" } td { "Original read matches; upload absent" } }
                            tr { th scope="row" { "Dufs: upload enabled, writer scoped to inbox" } td { "Writer PUT 201; reader GET 200" } td { "Both checks match writer-v1 bytes" } }
                            tr { th scope="row" { "Same Dufs profile: reader PUT / writer outside-path PUT" } td { "403 / 403" } td { "Neither target created" } }
                            tr { th scope="row" { "Same Dufs profile: writer DELETE" } td { "403" } td { "Uploaded bytes remain" } }
                            tr { th scope="row" { "miniserve: auth and read-only DAV" } td { "Anonymous GET 401; account GET 200" } td { "Original file unchanged; authorized body matches" } }
                            tr { th scope="row" { "Same miniserve profile: PROPFIND / PUT" } td { "207 / 404" } td { "Listing contains sample; PUT creates nothing" } }
                            tr { th scope="row" { "miniserve: auth, DAV and inbox browser uploads" } td { "Writer POST 303; reader POST 303" } td { "Both accounts create their exact sample bytes" } }
                            tr { th scope="row" { "Same upload profile: DAV PUT" } td { "404" } td { "No DAV target created" } }
                        }
                    }
                }
                p {
                    "The miniserve 303 responses redirect after multipart uploads. The lab deliberately does not follow them: "
                    "a later directory page returning 200 would describe the redirect destination, not the upload response. "
                    "The bytes on disk establish that the accepted upload happened. Conversely, the 404 on PUT is an observed result for this route and version, "
                    "not a universal status-code requirement for every read-only server."
                }
                p {
                    "The Dufs path test is a sibling file under the disposable served root. It is not a path-traversal attack, "
                    "an outside-root symlink test or evidence that every filesystem boundary is enforced. "
                    "The miniserve reader name is intentionally misleading: it demonstrates why a label must not stand in for a permission test."
                }
            }
            section aria-labelledby="worksheet" {
                h2 id="worksheet" { "Turn your sharing policy into an acceptance contract" }
                p {
                    "Use the " a href="/assets/examples/file-server-v1/acceptance.tsv" download="acceptance.tsv" { "downloadable acceptance worksheet" }
                    " before adapting a command. It separates identity, path, interface, expected result and observed file state. "
                    "Mark each row allow, deny or not needed before running it. An untested requirement stays untested; it does not inherit a pass from the sample lab."
                }
                ol {
                    li { "Name the smallest share root and the intended identities. Keep the release artifact separate from the inbox if they have different write policies." }
                    li { "For each identity, test a direct download as well as a listing. A visible name and readable contents are separate observations." }
                    li { "Test one allowed upload and one denied upload into another path. Read back the allowed file, then inspect the denied target for unintended changes." }
                    li { "Decide overwrite and deletion behavior explicitly. Our lab tests denied Dufs deletion, but does not establish either tool's complete overwrite policy." }
                    li { "Use the real browser or DAV client for a second acceptance pass. Browser authentication prompts, client retries and reverse-proxy paths are outside this Python HTTP exercise." }
                    li { "Stop the server after the trial. Check that the former share address is no longer reachable before considering the experiment finished." }
                }
                p {
                    "For a one-team release share, a successful miniserve trial may be enough to choose the application, while remote deployment still needs separate review. "
                    "For mixed reader/writer roles, use Dufs's path rules and repeat both positive and negative cases for each identity. "
                    "If your requirement is an upload-only drop box, add direct-read denials to the contract. Neither our read-write Dufs profile nor our shared-upload miniserve profile demonstrates that requirement."
                }
            }
            section aria-labelledby="limits" {
                h2 id="limits" { "Boundaries this comparison does not certify" }
                p {
                    "Loopback HTTP checks application behavior in a disposable folder. They do not validate TLS, Internet exposure, "
                    "container mounts, proxy authentication, concurrent clients, large uploads or recovery after a crash. "
                    "Do not publish the lab's local setup as a production deployment recipe. Enable only required operations, "
                    "and review the installed release's documentation and current maintainer notices before exposing real files."
                }
                p {
                    "Archives and symlinks need their own boundary tests. On November 3, 2026, miniserve's "
                    a href="https://github.com/svenstaro/miniserve/issues/1568" { "archive/symlink issue report" }
                    " and " a href="https://github.com/svenstaro/miniserve/pull/1573" { "proposed fix" }
                    " were still open. We did not reproduce that report, and our profiles leave archive generation disabled. "
                    "Do not infer archive containment from a passing ordinary download or upload check."
                }
                p {
                    "Dufs's " a href="https://github.com/sigoden/dufs/blob/v0.47.0/SECURITY.md" { "security policy" }
                    " explains that some unauthenticated DAV listings expose protected entry names without granting file access. "
                    "It also warns that served active content runs on the file server's origin. "
                    "Keep untrusted uploads away from trusted application origins; a name hidden from a listing is not the access-control test you need."
                }
                p {
                    "This is a functional comparison of tiny synthetic fixtures, not a vulnerability assessment, throughput benchmark or endorsement of a deployment. "
                    "The original contribution is the paired-response-and-state method and reusable policy worksheet. "
                    "Keep the observations attached to their versions, then rerun the checks that matter when you upgrade."
                }
            }
            section aria-labelledby="questions" {
                h2 id="questions" { "Common Dufs and miniserve questions" }
                @for (question, answer) in &faqs { h3 { (question) } p { (answer) } }
            }
            aside class="guide-note" {
                p { "Choose the application by the policy you can demonstrate: " a href="/projects/miniserve" { "evaluate miniserve for a shared browser workflow" }
                    " or " a href="/projects/dufs" { "evaluate Dufs for account-scoped writable paths" } ". Keep the worksheet with your trial results." }
            }
            script type="application/ld+json" { (PreEscaped(schema.to_string())) }
        }
    }
}
