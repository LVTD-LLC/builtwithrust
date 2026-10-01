use super::{check_icon, external_icon, format_stars, project_grid};
use crate::db::Project;
use maud::{Markup, html};

pub fn show(p: &Project, related: &[Project]) -> Markup {
    let stack = p.stack_list();
    html! {
        article class="project" {
            div class="project-head" {
                img class="logo logo-lg" src=(p.logo()) alt="" width="56" height="56" decoding="async";
                div {
                    h1 {
                        (p.name)
                        @if p.is_verified() { span class="badge-verified" title="Verified: confirmed built with Rust" { (check_icon()) } }
                        @if p.is_featured() { span class="pill pill-featured" { "Featured" } }
                    }
                    p class="tagline-lg" { (p.tagline) }
                }
                div class="project-actions" {
                    a class="btn btn-primary" href=(p.outbound_url()) rel="noopener nofollow" target="_blank" { "Visit " (p.domain()) (external_icon()) }
                    @if let Some(repo) = &p.repo_url {
                        a class="btn" href=(repo) rel="noopener nofollow" target="_blank" { "Repository" (external_icon()) }
                    }
                }
            }
            div class="project-body" {
                div class="prose" {
                    @if p.description.is_empty() {
                        p { (p.name) " is built with Rust." }
                    } @else {
                        @for para in p.description.split("\n\n") { p { (para.trim()) } }
                    }
                }
                aside class="project-meta" {
                    dl {
                        dt { "Stars" } dd { (format_stars(p.stars)) }
                        dt { "License" } dd { (p.license.as_deref().unwrap_or("Unknown")) }
                        dt { "Category" }
                        dd {
                            @match (&p.category_slug, &p.category_name) {
                                (Some(s), Some(n)) => a href={ "/categories/" (s) } { (n) },
                                _ => "—",
                            }
                        }
                        dt { "Listed" } dd { (p.created_at.get(..10).unwrap_or(&p.created_at)) }
                    }
                    @if !stack.is_empty() {
                        h3 { "Rust stack" }
                        div class="pills" { @for s in &stack { span class="pill" { (s) } } }
                    }
                    @if !p.is_featured() {
                        a class="btn btn-small" href="/feature" { "Feature this project" }
                    }
                }
            }
            @if !related.is_empty() {
                section class="related" {
                    h2 { "More like this" }
                    (project_grid(related))
                }
            }
        }
    }
}
