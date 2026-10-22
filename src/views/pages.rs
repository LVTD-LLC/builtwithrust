use super::{external_icon, project_grid};
use crate::db::{CategoryCount, Project};
use maud::{Markup, html};

pub struct HomeData<'a> {
    pub developer_guide_site_url: Option<&'a str>,
    pub data_guide_site_url: Option<&'a str>,
    pub database_guide_site_url: Option<&'a str>,
    pub infrastructure_guide_site_url: Option<&'a str>,
    pub document_guide_site_url: Option<&'a str>,
    pub browse: &'a crate::directory::Browse,
    pub facets: &'a crate::directory::Facets,
    pub projects: &'a [Project],
    pub categories: &'a [CategoryCount],
    pub total: i64,
    pub newsletter_enabled: bool,
    pub query: Option<&'a str>,
    pub active_category: Option<&'a CategoryCount>,
}

pub fn home(d: HomeData<'_>) -> Markup {
    let heading = match (d.query, d.active_category) {
        (Some(q), _) => format!("Results for “{q}”"),
        (None, Some(c)) => format!("{} built with Rust", c.name),
        (None, None) => "Everything built with Rust".to_string(),
    };
    html! {
        @if d.query.is_none() && d.active_category.is_none() {
            section class="hero" {
                span class="eyebrow" { "Itself built with Rust" }
                h1 { "Websites, apps and tools" br; "built with Rust" }
                p class="lede" {
                    "A curated directory of " strong { (d.total) } " real products shipping Rust in production. "
                    "Find who uses it, what they built, and which crates they built it with."
                }

            }
        }
        @if d.newsletter_enabled && d.query.is_none() && d.active_category.is_none() {
            (newsletter_form(None, ""))
        }
        section class="listing" id="projects" {
            div class="listing-head" {
                @if d.query.is_some() || d.active_category.is_some() {
                    h1 { (heading) }
                } @else {
                    h2 { (heading) }
                }
                @if !d.browse.is_default() {
                    a class="muted" href="/#projects" { "Clear filters" }
                }
            }
            @if let Some(site_url) = d.developer_guide_site_url {
                (super::developer_guide::intro())
                (super::developer_guide::javascript_workflows(site_url, d.projects))
            }
            @if let Some(site_url) = d.data_guide_site_url {
                (data_workflow_links(site_url))
            }
            @if let Some(site_url) = d.database_guide_site_url {
                (database_workflow_links(site_url))
            }
            @if let Some(site_url) = d.infrastructure_guide_site_url {
                (network_workflow_links(site_url, d.projects))
                (file_sharing_workflow_links(site_url, d.projects))
            }
            @if let Some(site_url) = d.document_guide_site_url {
                (super::document_guide::workflows(site_url, d.projects))
            }
            @if d.query.is_none() && d.active_category.is_none() && d.browse.is_default() {
                nav class="pills category-links" aria-label="Browse projects by category" {
                    @for c in d.categories {
                        a class="pill" href={ "/categories/" (c.slug) } { (c.name) " (" (c.count) ")" }
                    }
                }
            }
            form class="browse" action="/#projects" method="get" role="search" aria-label="Filter projects" {
                div class="browse-primary" {
                    label class="browse-search" { "Search projects"
                        input type="search" name="q" value=(d.browse.q) maxlength="100" placeholder="Name, stack or description…";
                    }
                    label { "Category"
                        select name="category" {
                            option value="" { "All categories" }
                            @for c in d.categories {
                                option value=(c.slug) selected[d.browse.category == c.slug] { (c.name) " (" (c.count) ")" }
                            }
                            @if !d.browse.category.is_empty() && d.active_category.is_none() {
                                option value=(d.browse.category) selected { (d.browse.category) " (unavailable)" }
                            }
                        }
                    }
                    label { "Sort / discover"
                        select name="sort" {
                            @for (value, label) in [("", "Recommended"), ("stars", "Most starred"), ("newest", "Recently added"), ("name", "Name A–Z"), ("gems", "Hidden gems · under 10k stars")] {
                                option value=(value) selected[d.browse.sort == value] { (label) }
                            }
                        }
                    }
                }
                details class="browse-more" open[!d.browse.stack.is_empty() || !d.browse.license.is_empty() || !d.browse.source.is_empty() || !d.browse.stars.is_empty() || !d.browse.verified.is_empty()] {
                    summary { "Refine by stack, license and more" }
                    div class="browse-secondary" {
                        (facet_select("stack", "Crate / stack", "Any stack", &d.facets.stacks, &d.browse.stack))
                        (facet_select("license", "License", "Any license", &d.facets.licenses, &d.browse.license))
                        label { "Repository"
                            select name="source" {
                                @for (value, label) in [("", "Any project"), ("available", "Repository linked"), ("unlisted", "No repository listed")] {
                                    option value=(value) selected[d.browse.source == value] { (label) }
                                }
                            }
                        }
                        label { "GitHub stars"
                            select name="stars" {
                                @for (value, label) in [("", "Any star count"), ("under-1000", "Under 1,000"), ("1000", "1,000 or more"), ("10000", "10,000 or more")] {
                                    option value=(value) selected[d.browse.stars == value] { (label) }
                                }
                            }
                        }
                    }
                    label class="browse-check" {
                        input type="checkbox" name="verified" value="1" checked[d.browse.verified == "1"];
                        "Verified Rust projects only"
                    }
                }
                div class="browse-actions" {
                    button class="btn btn-primary" type="submit" { "Apply filters" }
                    span class="small muted" { "Bookmark or share the URL to save this view." }
                }
            }
            p class="browse-results" role="status" {
                strong { (d.projects.len()) } " of " (d.total) " projects"
                @if d.browse.sort == "gems" {
                    " · Known star counts below 10,000, smallest first."
                } @else if d.browse.sort.is_empty() {
                    " · Featured first, then most starred."
                } @else if d.browse.sort == "newest" {
                    " · Newest directory additions first."
                }
            }
            @if !d.projects.is_empty() && (!d.browse.stars.is_empty() || d.browse.sort == "stars" || d.browse.sort == "gems") {
                p class="small muted browse-note" { "Stars are catalog snapshots, not live counts. Unknown counts are excluded from star filters." }
            }
            @if d.projects.is_empty() && !d.browse.is_default() {
                p class="browse-empty" { "No projects match this combination. Try removing a filter or " a href="/#projects" { "reset all filters" } "." }
            }
            (project_grid(d.projects))
        }
        @if let Some(site_url) = d.developer_guide_site_url {
            (super::developer_guide::body(site_url))
        }
        @if d.browse.is_default() && d.active_category.is_none() {
            section class="task-guide" aria-labelledby="python-guide-title" {
                h2 id="python-guide-title" { "Rust-powered tools for Python workflows" }
                p { "Find the right tool for project management, linting or DataFrame work." }
                a href="/guides/rust-python-tools" { "Compare uv, Ruff and Polars by task →" }
            }
            section class="task-guide" aria-labelledby="terminal-guide-title" {
                h2 id="terminal-guide-title" { "Choose your next terminal improvement" }
                p { "Compare a terminal emulator, shell, history tool and file manager by the job you want to improve." }
                a href="/guides/rust-terminal-tools" { "Compare Alacritty, Nushell, Atuin and Yazi by task →" }
            }
            section class="task-guide" aria-labelledby="editor-guide-title" {
                h2 id="editor-guide-title" { "Choose a Rust-built editor for your workflow" }
                p { "Compare language support, remote development and collaboration before changing your everyday editor." }
                a href="/guides/zed-vs-lapce" { "Compare Zed and Lapce with a reversible trial →" }
            }
        }
    }
}

fn file_sharing_workflow_links(site_url: &str, projects: &[Project]) -> Markup {
    let choices = [
        (
            "miniserve",
            "Share a folder through the browser →",
            "Start with miniserve for an ad-hoc HTTP share. It also supports authentication and optional browser uploads. Its WebDAV option is read-only: browsing a mounted share does not mean the client can write to it.",
        ),
        (
            "dufs",
            "Evaluate writable WebDAV and account scopes →",
            "Start with Dufs when a WebDAV client needs to write files or accounts need different path permissions. An account’s read-write rule still cannot enable uploads unless the server’s global upload permission allows them.",
        ),
    ];
    let available: Vec<_> =
        choices.into_iter().filter(|(slug, _, _)| projects.iter().any(|p| p.slug == *slug)).collect();
    if available.is_empty() {
        return html! {};
    }
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{site_url}/categories/infrastructure#file-sharing-workflows"),
        "name": "Choose a Rust file server by how people will use the share",
        "dateModified": "2026-10-19",
        "isPartOf": { "@type": "CollectionPage", "@id": format!("{site_url}/categories/infrastructure") }
    });
    html! {
        section id="file-sharing-workflows" aria-labelledby="file-sharing-workflows-title" {
            h2 id="file-sharing-workflows-title" { "Choose a Rust file server by how people will use the share" }
            p class="guide-intro" {
                "For a temporary browser-based file share, evaluate miniserve. For writable WebDAV or account rules scoped to different paths, evaluate Dufs. "
                "Both support authentication and optional browser uploads. Check the access method before choosing: miniserve’s WebDAV support is read-only, "
                "while Dufs write access also depends on enabled server operations."
            }
            nav class="workflow-guides" aria-label="File servers by sharing task" {
                @for (slug, title, detail) in available {
                    a href={ "/projects/" (slug) } {
                        strong { (title) }
                        span { (detail) }
                    }
                }
            }
            details {
                summary { "Try the access policy with a disposable folder" }
                ol {
                    li { "Use non-sensitive sample files in a dedicated folder. Bind the first trial to loopback; a local test does not verify remote access or TLS." }
                    li { "Choose browser downloads, browser uploads or a WebDAV client. Test that exact path; a browser upload does not prove WebDAV writes work." }
                    li { "If authentication is required, confirm an unauthenticated request is denied and the intended account can download the sample." }
                    li { "Attempt an upload with each intended read-only and write-enabled account. Confirm denied writes leave the folder unchanged; enable only the operations you need." }
                    li { "Stop the server, confirm the share is unreachable, and review uploaded files before keeping them. Check TLS, network exposure and client compatibility separately before sharing remotely." }
                }
            }
            p class="guide-note" {
                "These are file-serving tools, not a tested replacement for your backup or synchronization workflow. "
                "Check the official " a href="https://github.com/svenstaro/miniserve#usage" { "miniserve usage and options" }
                " and " a href="https://github.com/sigoden/dufs#access-control" { "Dufs access-control rules" } " for your installed version."
            }
            p class="small muted" { "Sources checked " time datetime="2026-10-19" { "October 19, 2026" } "." }
        }
        script type="application/ld+json" { (maud::PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}

fn network_workflow_links(site_url: &str, projects: &[Project]) -> Markup {
    let choices = [
        (
            "gping",
            "Compare host latency →",
            "gping graphs ping times for multiple hosts in the terminal. Start here to compare how latency changes over time.",
        ),
        (
            "trippy",
            "Inspect the route to a destination →",
            "Trippy combines traceroute and ping to inspect individual network hops. Check its platform-specific privilege requirements before running it.",
        ),
        (
            "sniffnet",
            "See what traffic your computer exchanges →",
            "Sniffnet is a graphical traffic monitor with connection inspection and PCAP import/export. Check capture dependencies and permissions for your operating system.",
        ),
        (
            "pingora",
            "Build a custom proxy service →",
            "Pingora is a Rust framework for programmable network services and HTTP proxies. It is a development building block, not a ready-to-run latency diagnostic.",
        ),
    ];
    // Catalog writes can unpublish a project. Never keep a dead recommendation in the cached hub.
    let available: Vec<_> =
        choices.into_iter().filter(|(slug, _, _)| projects.iter().any(|p| p.slug == *slug)).collect();
    if available.is_empty() {
        return html! {};
    }
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "CollectionPage",
        "name": "Infrastructure built with Rust",
        "url": format!("{site_url}/categories/infrastructure"),
        "hasPart": {
            "@type": "WebPageElement",
            "name": "Choose a network tool by the question you need to answer",
            "dateModified": "2026-10-18",
            "url": format!("{site_url}/categories/infrastructure#network-workflows")
        }
    });
    html! {
        section id="network-workflows" aria-labelledby="network-workflows-title" {
            h2 id="network-workflows-title" { "Choose a network tool by the question you need to answer" }
            p class="guide-intro" {
                "Rust network tools serve different jobs: gping graphs host latency, Trippy inspects network routes, "
                "Sniffnet shows traffic on your computer, and Pingora helps developers build proxy services. "
                "Choose the question first; a ping graph, a packet capture and a proxy framework are not interchangeable tests of application health."
            }
            nav class="workflow-guides" aria-label="Network tools by task" {
                @for (slug, title, detail) in available {
                    a href={ "/projects/" (slug) } {
                        strong { (title) }
                        span { (detail) }
                    }
                }
            }
            p class="guide-note" {
                "For a diagnostic trial, pick a destination or interface you own or are authorized to inspect, "
                "write down the symptom and collect a short baseline before changing anything. "
                "Keep captures private and review them before sharing. A diagnostic observation is a starting point, not proof of the cause. "
                "For Pingora, start with the official development guide instead."
            }
            p class="guide-note" {
                "Check the official " a href="https://github.com/orf/gping" { "gping usage" } ", "
                a href="https://trippy.rs/" { "Trippy documentation" } ", "
                a href="https://github.com/GyulyVGC/sniffnet" { "Sniffnet setup" } " and "
                a href="https://github.com/cloudflare/pingora" { "Pingora framework guide" } " before choosing a tool."
            }
            p class="small muted" { "Sources checked " time datetime="2026-10-18" { "October 18, 2026" } "." }
        }
        script type="application/ld+json" { (maud::PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}

fn database_workflow_links(site_url: &str) -> Markup {
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "CollectionPage",
        "name": "Databases & Search built with Rust",
        "url": format!("{site_url}/categories/databases-search"),
        "hasPart": {
            "@type": "WebPageElement",
            "name": "Search existing content or model application data?",
            "dateModified": "2026-10-17",
            "url": format!("{site_url}/categories/databases-search#database-workflows")
        }
    });
    html! {
        section id="database-workflows" aria-labelledby="database-workflows-title" {
            h2 id="database-workflows-title" { "Search existing content or model application data?" }
            p class="guide-intro" {
                "Start with the job, not the implementation language. Meilisearch indexes content for search interfaces; "
                "SurrealDB stores and queries application data across several models. Both support full-text search, so the distinction is not search versus no search."
            }
            nav class="workflow-guides" aria-label="Database and search projects" {
                a href="/projects/meilisearch" {
                    strong { "Add search to content you already have →" }
                    span { "Explore Meilisearch for typo-tolerant search, filtering and faceted navigation over indexed documents." }
                }
                a href="/projects/surrealdb" {
                    strong { "Store records and query their relationships →" }
                    span { "Explore SurrealDB for document and graph data, with integrated full-text search in the same database." }
                }
            }
            p class="guide-note" {
                "Before adopting a tool, try a disposable copy of representative data: check missing and updated records, "
                "access restrictions and the queries your app needs. For a separate search index, also plan how source changes and deletions reach it. "
                "These are evaluation checks, not benchmark results."
            }
            p class="guide-note" {
                "Read the " a href="https://www.meilisearch.com/docs/getting_started/overview" { "Meilisearch search overview" }
                " and " a href="https://surrealdb.com/docs/learn/data-models/full-text-search/overview" { "SurrealDB full-text search model" }
                ". Check edition-specific licenses and supported deployment options before committing to either."
            }
            p class="small muted" { "Sources checked " time datetime="2026-10-17" { "October 17, 2026" } "." }
        }
        script type="application/ld+json" { (maud::PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}

fn data_workflow_links(site_url: &str) -> Markup {
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "CollectionPage",
        "name": "Data & AI built with Rust",
        "url": format!("{site_url}/categories/data-and-ai"),
        "hasPart": {
            "@type": "WebPageElement",
            "name": "Choose a data workflow",
            "dateModified": "2026-10-16",
            "url": format!("{site_url}/categories/data-and-ai#data-workflows")
        }
    });
    html! {
        section id="data-workflows" aria-labelledby="data-workflows-title" {
            h2 id="data-workflows-title" { "Choose a data workflow" }
            p class="guide-intro" {
                "Start with where you want to work: a shell pipeline or a DataFrame program. "
                "qsv and Polars overlap—qsv’s optional SQL commands use Polars—so this is a workflow choice, not a ranking of competing engines."
            }
            nav class="workflow-guides" aria-label="Data workflow projects" {
                a href="/projects/qsv" {
                    strong { "Clean tabular files in the terminal →" }
                    span { "Explore qsv for selecting columns, filtering rows, joining files and checking data in command-line pipelines." }
                }
                a href="/projects/polars" {
                    strong { "Build a DataFrame workflow →" }
                    span { "Explore Polars for expressions, transformations and lazy queries in Python or Rust code." }
                }
            }
            p class="guide-note" {
                "Check the " a href="https://github.com/dathere/qsv#installation" { "qsv build’s available commands" }
                " before choosing it: features vary by build. For Polars, start with the "
                a href="https://docs.pola.rs/user-guide/getting-started/" { "official DataFrame guide" }
                ". Try a copy of one representative file and check types, missing values and output rows before changing a working pipeline."
            }
            p class="guide-jump" {
                a href="/guides/rust-python-tools" { "Working in Python? See where Polars fits alongside uv and Ruff →" }
            }
            p class="small muted" { "Sources checked " time datetime="2026-10-16" { "October 16, 2026" } "." }
        }
        script type="application/ld+json" { (maud::PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}

fn facet_select(name: &str, label: &str, all: &str, values: &[String], selected: &str) -> Markup {
    html! {
        label { (label)
            select name=(name) {
                option value="" { (all) }
                @for value in values {
                    option value=(value) selected[value.eq_ignore_ascii_case(selected)] { (value) }
                }
                @if !selected.is_empty() && !values.iter().any(|v| v.eq_ignore_ascii_case(selected)) {
                    option value=(selected) selected { (selected) " (unavailable)" }
                }
            }
        }
    }
}

pub fn categories(cats: &[CategoryCount]) -> Markup {
    html! {
        section class="page" {
            h1 { "Categories" }
            p class="lede" { "Browse Rust-built projects by what they do." }
            div class="grid" {
                @for c in cats {
                    a class="card card-compact" href={ "/categories/" (c.slug) } {
                        h3 { (c.name) }
                        p class="muted" { (c.count) " project" @if c.count != 1 { "s" } }
                    }
                }
            }
        }
    }
}

pub fn submit_form(error: Option<&str>, url: &str) -> Markup {
    html! {
        section class="page narrow" {
            h1 { "Submit a site built with Rust" }
            p class="lede" {
                "Know a website, app or tool that runs on Rust? Send us the URL. "
                "Every submission is reviewed before it is listed."
            }
            @if let Some(e) = error { p class="alert" { (e) } }
            form class="form" method="post" action="/submit" {
                label { "Website URL" span class="req" { "*" }
                    input type="url" name="url" required placeholder="https://example.com" value=(url);
                }
                label { "Your email " span class="muted" { "(optional, so we can tell you when it is listed)" }
                    input type="email" name="email" placeholder="you@example.com";
                }
                label { "Anything we should know? " span class="muted" { "(optional)" }
                    textarea name="note" rows="4" placeholder="How do you know it is built with Rust? Which parts? Any links to blog posts or repos help."{}
                }
                // Honeypot: real users never see or fill this.
                div class="hp" aria-hidden="true" { input type="text" name="website" tabindex="-1" autocomplete="off"; }
                button class="btn btn-primary" type="submit" { "Submit for review" }
            }
        }
    }
}

pub fn submit_thanks() -> Markup {
    html! {
        section class="page narrow" {
            h1 { "Thanks, we got it." }
            p class="lede" { "We review every submission by hand. If it checks out, it will show up in the directory." }
            a class="btn" href="/" { "Back to the directory" }
        }
    }
}

pub fn feature(projects: &[Project], payments_enabled: bool, feature_days: i64, error: Option<&str>) -> Markup {
    html! {
        section class="page narrow" {
            h1 { "Feature your project" }
            p class="lede" {
                "Put your project at the top of the homepage and first in its category for " (feature_days) " days. "
                "One flat payment, no subscription."
            }
            div class="pricing" {
                ul class="checks" {
                    li { "Pinned to the top of the homepage" }
                    li { "First in its category and in search results" }
                    li { "“Featured” badge on the card and project page" }
                    li { "Runs for " (feature_days) " days from payment" }
                }
            }
            @if let Some(e) = error { p class="alert" { (e) } }
            @if payments_enabled {
                form class="form" method="post" action="/api/checkout" {
                    label { "Project" span class="req" { "*" }
                        select name="project_slug" required {
                            option value="" { "Choose a listed project…" }
                            @for p in projects { option value=(p.slug) { (p.name) } }
                        }
                    }
                    label { "Receipt email" span class="req" { "*" }
                        input type="email" name="email" required placeholder="you@example.com";
                    }
                    button class="btn btn-primary" type="submit" { "Continue to payment" }
                    p class="muted small" { "Payments are handled by Stripe. Your project must already be listed; " a href="/submit" { "submit it first" } " if it is not." }
                }
            } @else {
                p class="alert" { "Featured slots are not open yet. Check back soon." }
            }
        }
    }
}

pub fn feature_success(project: Option<&Project>) -> Markup {
    html! {
        section class="page narrow" {
            h1 { "You're featured. Thank you!" }
            @if let Some(p) = project {
                p class="lede" { a href={ "/projects/" (p.slug) } { (p.name) } " is now featured. It can take a few seconds for the badge to appear." }
            } @else {
                p class="lede" { "Your payment went through. Your project will show as featured within a few seconds." }
            }
            a class="btn" href="/" { "See the homepage" }
        }
    }
}

pub fn feature_cancel() -> Markup {
    html! {
        section class="page narrow" {
            h1 { "No charge was made." }
            p class="lede" { "You can come back any time." }
            a class="btn" href="/feature" { "Try again" }
        }
    }
}

pub fn not_found() -> Markup {
    html! {
        section class="page narrow" {
            h1 { "Not found" }
            p class="lede" { "That page does not exist. The directory does, though." }
            a class="btn" href="/" { "Go home" (external_icon()) }
        }
    }
}

pub fn newsletter_form(error: Option<&str>, email: &str) -> Markup {
    html! {
        section class="newsletter" aria-labelledby="newsletter-title" {
            div {
                h2 id="newsletter-title" { "Weekly Rust news and projects" }
                p { "New projects, useful crates, and news from the Rust community. One email a week." }
            }
            form method="post" action="/newsletter" class="newsletter-form" {
                @if let Some(error) = error { p class="alert" role="alert" { (error) } }
                label for="newsletter-email" { "Email address" }
                div class="newsletter-fields" {
                    input id="newsletter-email" name="email" type="email" required maxlength="254" autocomplete="email" placeholder="you@example.com" value=(email);
                    button class="btn btn-primary" type="submit" { "Subscribe" }
                }
                div class="hp" aria-hidden="true" { input type="text" name="website" tabindex="-1" autocomplete="off"; }
                p class="small" { "Confirm by email to join. Unsubscribe anytime." }
            }
        }
    }
}
