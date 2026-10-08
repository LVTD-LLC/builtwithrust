use super::{external_icon, project_grid};
use crate::db::{CategoryCount, Project};
use maud::{Markup, html};

pub struct HomeData<'a> {
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
