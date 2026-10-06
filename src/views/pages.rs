use super::{external_icon, project_grid};
use crate::db::{CategoryCount, Project};
use maud::{Markup, html};

pub struct HomeData<'a> {
    pub projects: &'a [Project],
    pub categories: &'a [CategoryCount],
    pub total: i64,
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
                form class="search" action="/" method="get" role="search" {
                    input type="search" name="q" placeholder="Search by name, stack or description…" aria-label="Search projects" autocomplete="off";
                    button class="btn btn-primary" type="submit" { "Search" }
                }
            }
        }
        section class="listing" {
            div class="listing-head" {
                h2 { (heading) }
                @if d.query.is_some() || d.active_category.is_some() {
                    a class="muted" href="/" { "Clear filters" }
                }
            }
            @if !d.categories.is_empty() {
                nav class="chips" aria-label="Categories" {
                    a class={ "chip" @if d.active_category.is_none() { " active" } } href="/" { "All" }
                    @for c in d.categories {
                        a class={ "chip" @if d.active_category.is_some_and(|a| a.id == c.id) { " active" } }
                          href={ "/categories/" (c.slug) } { (c.name) span class="count" { (c.count) } }
                    }
                }
            }
            (project_grid(d.projects))
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
