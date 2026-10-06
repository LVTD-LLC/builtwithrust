//! HTML templates (maud, compile-time checked). `layout` wraps every page.

pub mod pages;
pub mod project;

use crate::AppState;
use crate::db::Project;
use maud::{DOCTYPE, Markup, PreEscaped, html};

pub struct Page<'a> {
    pub title: &'a str,
    pub description: &'a str,
    /// Path (starting with `/`) used for the canonical URL.
    pub path: &'a str,
    pub body: Markup,
}

pub fn layout(state: &AppState, page: Page<'_>) -> Markup {
    let canonical = format!("{}{}", state.cfg.site_url, page.path);
    let full_title = if page.path == "/" {
        "Built with Rust — websites, apps and tools made with Rust".to_string()
    } else {
        format!("{} — Built with Rust", page.title)
    };
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (full_title) }
                meta name="description" content=(page.description);
                link rel="canonical" href=(canonical);
                link rel="icon" href=(state.asset_url("favicon.svg")) type="image/svg+xml";
                link rel="stylesheet" href=(state.asset_url("style.css"));
                meta property="og:title" content=(full_title);
                meta property="og:description" content=(page.description);
                meta property="og:url" content=(canonical);
                meta property="og:type" content="website";
                meta name="twitter:card" content="summary";
                // Apply the saved theme before first paint to avoid a flash.
                script { (PreEscaped(THEME_BOOT)) }
                @if let Some(ph) = state.posthog.config() {
                    script { (PreEscaped(posthog_snippet(&ph.key, &ph.host))) }
                }
            }
            body {
                header class="site-header" {
                    div class="container header-inner" {
                        a class="brand" href="/" {
                            (crab_icon())
                            span { "Built with Rust" }
                        }
                        nav class="nav" {
                            a href="/" { "Projects" }
                            a href="/categories" { "Categories" }
                            a href="/feature" { "Feature your project" }
                        }
                        div class="header-actions" {
                            form class="search-mini" action="/" method="get" role="search" {
                                input type="search" name="q" placeholder="Search" aria-label="Search projects";
                            }
                            button class="icon-btn" id="theme-toggle" type="button" aria-label="Toggle theme" {
                                (sun_icon()) (moon_icon())
                            }
                            a class="btn btn-primary" href="/submit" { "Submit" }
                        }
                    }
                }
                main class="container" { (page.body) }
                footer class="site-footer" {
                    div class="container footer-inner" {
                        div {
                            a class="brand" href="/" { (crab_icon()) span { "Built with Rust" } }
                            p class="muted" { "A curated directory of websites, apps and tools built with Rust. Itself built with Rust." }
                        }
                        div class="footer-links" {
                            a href="/" { "Projects" }
                            a href="/categories" { "Categories" }
                            a href="/submit" { "Submit a site" }
                            a href="/feature" { "Feature your project" }
                            a href="https://github.com/LVTD-LLC/builtwithrust" rel="noopener" { "Source" }
                        }
                    }
                }
                script { (PreEscaped(THEME_TOGGLE)) }
            }
        }
    }
}

const THEME_BOOT: &str = r#"(function(){try{var t=localStorage.getItem('theme');if(t==='dark'||(!t&&matchMedia('(prefers-color-scheme: dark)').matches))document.documentElement.dataset.theme='dark';}catch(e){}})();"#;

const THEME_TOGGLE: &str = r#"document.getElementById('theme-toggle').addEventListener('click',function(){var d=document.documentElement.dataset.theme==='dark';document.documentElement.dataset.theme=d?'light':'dark';try{localStorage.setItem('theme',d?'light':'dark')}catch(e){}});"#;

fn posthog_snippet(key: &str, host: &str) -> String {
    format!(
        r#"!function(t,e){{var o,n,p,r;e.__SV||(window.posthog=e,e._i=[],e.init=function(i,s,a){{function g(t,e){{var o=e.split(".");2==o.length&&(t=t[o[0]],e=o[1]),t[e]=function(){{t.push([e].concat(Array.prototype.slice.call(arguments,0)))}}}}(p=t.createElement("script")).type="text/javascript",p.crossOrigin="anonymous",p.async=!0,p.src=s.api_host.replace(".i.posthog.com","-assets.i.posthog.com")+"/static/array.js",(r=t.getElementsByTagName("script")[0]).parentNode.insertBefore(p,r);var u=e;for(void 0!==a?u=e[a]=[]:a="posthog",u.people=u.people||[],u.toString=function(t){{var e="posthog";return"posthog"!==a&&(e+="."+a),t||(e+=" (stub)"),e}},u.people.toString=function(){{return u.toString(1)+".people (stub)"}},o="init capture register register_once register_for_session unregister unregister_for_session getFeatureFlag getFeatureFlagPayload isFeatureEnabled reloadFeatureFlags updateEarlyAccessFeatureEnrollment getEarlyAccessFeatures on onFeatureFlags onSessionId getSurveys getActiveMatchingSurveys renderSurvey canRenderSurvey identify setPersonProperties group resetGroups setPersonPropertiesForFlags resetPersonPropertiesForFlags setGroupPropertiesForFlags resetGroupPropertiesForFlags reset get_distinct_id getGroups get_session_id get_session_replay_url alias set_config startSessionRecording stopSessionRecording sessionRecordingStarted captureException loadToolbar get_property getSessionProperty createPersonProfile opt_in_capturing opt_out_capturing has_opted_in_capturing has_opted_out_capturing clear_opt_in_out_capturing debug getPageViewId captureTraceFeedback captureTraceMetric".split(" "),n=0;n<o.length;n++)g(u,o[n]);e._i.push([i,s,a])}},e.__SV=1)}}(document,window.posthog||[]);
posthog.init('{key}',{{api_host:'{host}',defaults:'2025-05-24',person_profiles:'identified_only'}});"#
    )
}

// ---------------------------------------------------------------------------
// Shared components
// ---------------------------------------------------------------------------

pub fn project_card(p: &Project) -> Markup {
    html! {
        a class={ "card" @if p.is_featured() { " card-featured" } } href={ "/projects/" (p.slug) } {
            div class="card-head" {
                img class="logo" src=(p.logo()) alt="" width="32" height="32" loading="lazy" decoding="async";
                h3 { (p.name) }
                @if p.is_verified() { span class="badge-verified" title="Verified: confirmed built with Rust" { (check_icon()) } }
                @if p.is_featured() { span class="pill pill-featured" { "Featured" } }
            }
            p class="tagline" { (p.tagline) }
            ul class="stats" {
                li { span { (star_icon()) "Stars" } span class="leader" {} span class="value" { (format_stars(p.stars)) } }
                li { span { (scale_icon()) "License" } span class="leader" {} span class="value" { (p.license.as_deref().unwrap_or("Unknown")) } }
                li { span { (folder_icon()) "Category" } span class="leader" {} span class="value" { (p.category_name.as_deref().unwrap_or("—")) } }
            }
            @let stack = p.stack_list();
            @if !stack.is_empty() {
                div class="pills" {
                    @for s in stack.iter().take(4) { span class="pill" { (s) } }
                }
            }
        }
    }
}

pub fn project_grid(projects: &[Project]) -> Markup {
    html! {
        @if projects.is_empty() {
            p class="empty" { "Nothing here yet." }
        } @else {
            div class="grid" { @for p in projects { (project_card(p)) } }
        }
    }
}

pub fn format_stars(stars: Option<i64>) -> String {
    match stars {
        None => "—".to_string(),
        Some(n) if n >= 1000 => {
            let k = n as f64 / 1000.0;
            if k >= 100.0 { format!("{k:.0}k") } else { format!("{k:.1}k") }
        }
        Some(n) => n.to_string(),
    }
}

pub fn crab_icon() -> Markup {
    html! { svg class="icon brand-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" {
        path d="M12 2a5 5 0 0 1 5 5v2.3a7 7 0 1 1-10 0V7a5 5 0 0 1 5-5z";
        path d="M3 10l3 2M21 10l-3 2M4 16l3-1M20 16l-3-1M9 9h.01M15 9h.01";
    } }
}
pub fn check_icon() -> Markup {
    html! { svg class="icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" { path d="M12 2l2.4 2.1 3.1-.5 1 3 2.9 1.3-.6 3.1L22 13.5l-1.9 2.6.3 3.1-3.1.9-1.6 2.7L12 21.6l-3.7 1.2-1.6-2.7-3.1-.9.3-3.1L2 13.5l1.2-2.5-.6-3.1L5.5 6.6l1-3 3.1.5L12 2zm-1.2 13.4l5.6-5.6-1.4-1.4-4.2 4.2-2-2-1.4 1.4 3.4 3.4z"; } }
}
pub fn star_icon() -> Markup {
    html! { svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" aria-hidden="true" { path d="M12 3l2.8 5.7 6.2.9-4.5 4.4 1.1 6.2L12 17.3 6.4 20.2l1.1-6.2L3 9.6l6.2-.9L12 3z"; } }
}
pub fn scale_icon() -> Markup {
    html! { svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true" { path d="M12 3v18M4 7h16M6 7l-3 7a3 3 0 0 0 6 0L6 7zM18 7l-3 7a3 3 0 0 0 6 0l-3-7"; } }
}
pub fn folder_icon() -> Markup {
    html! { svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" aria-hidden="true" { path d="M3 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6z"; } }
}
pub fn sun_icon() -> Markup {
    html! { svg class="icon icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true" { circle cx="12" cy="12" r="4"; path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"; } }
}
pub fn moon_icon() -> Markup {
    html! { svg class="icon icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" aria-hidden="true" { path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"; } }
}
pub fn external_icon() -> Markup {
    html! { svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" { path d="M14 4h6v6M20 4l-9 9M19 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h5"; } }
}
