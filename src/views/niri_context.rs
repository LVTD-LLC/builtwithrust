use crate::db::Project;
use maud::{Markup, PreEscaped, html};

const XWAYLAND: &str =
    "https://github.com/niri-wm/niri/blob/689c1e914a5af782a476e433b79c15554bf5a312/docs/wiki/Xwayland.md";
const SOFTWARE: &str =
    "https://github.com/niri-wm/niri/blob/689c1e914a5af782a476e433b79c15554bf5a312/docs/wiki/Important-Software.md";
const SHARING: &str =
    "https://github.com/niri-wm/niri/blob/689c1e914a5af782a476e433b79c15554bf5a312/docs/wiki/Screencasting.md";

pub fn context(site_url: &str) -> Markup {
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPageElement",
        "@id": format!("{site_url}/projects/niri#niri-compatibility"),
        "name": "Will your applications fit a Niri desktop?",
        "dateModified": "2026-11-16",
        "isPartOf": { "@type": "WebPage", "@id": format!("{site_url}/projects/niri") }
    });
    html! {
        section class="task-guide prose" aria-labelledby="niri-compatibility" {
            h2 id="niri-compatibility" { "Will your applications fit a Niri desktop?" }
            p {
                "Niri manages local Wayland windows; this listing is not a remote-access service. "
                "Before switching desktops, check application launching, file dialogs and screen sharing separately. "
                "A working window does not prove that the rest of your workflow works."
            }
            ol {
                li {
                    strong { "Separate Wayland and X11 applications. " }
                    "List the applications you need and check their display backend. Since Niri 25.08, automatic X11 integration uses "
                    code { "xwayland-satellite" } " version 0.7 or newer, installed and available on the executable search path. "
                    "The " a href=(XWAYLAND) { "official Xwayland guidance" }
                    " also describes removing older manual startup and DISPLAY customizations when using automatic integration. "
                    "X11 tools that require fixed screen coordinates can need a nested compositor; do not assume every window behaves identically."
                }
                li {
                    strong { "Check desktop services, not just the compositor. " }
                    "Niri is not a complete desktop environment. Follow the " a href=(SOFTWARE) { "required-software checklist" }
                    " for portals, notifications and authentication prompts. In a trial session, open a harmless local file through an application's file picker "
                    "and confirm that dialogs and notifications work. Keep your existing desktop session available while evaluating the change."
                }
                li {
                    strong { "Treat sharing and remote control as separate requirements. " }
                    "For portal-based screencasting, the " a href=(SHARING) { "Niri screencasting documentation" }
                    " requires a working D-Bus session, PipeWire, the GNOME desktop portal and Niri running as a session. "
                    "Try a non-sensitive window in the application you actually use. A visible shared image does not verify remote keyboard or mouse input, "
                    "unattended access or reconnection; evaluate those requirements with your chosen remote-access tool separately."
                }
            }
            p {
                "Record each requirement as working, blocked or unverified, together with the application and installed versions. "
                "This is a suggested acceptance checklist, not a performed hardware, remote-client compatibility or security test. "
                a href="/categories/productivity" { "Browse other Rust productivity tools →" }
            }
            p class="small muted" { "Sources checked " time datetime="2026-11-16" { "November 16, 2026" } "; checklist by Built with Rust." }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}

pub fn discovery(projects: &[Project]) -> Markup {
    if !projects.iter().any(|project| project.slug == "niri") {
        return html! {};
    }
    html! {
        section id="desktop-workflows" aria-labelledby="desktop-workflows-title" {
            h2 id="desktop-workflows-title" { "Evaluate a scrolling Wayland desktop" }
            p { "Choosing Niri is a window-management decision. Check your required applications and desktop services before replacing a working session." }
            nav class="workflow-guides" aria-label="Wayland desktop evaluation" {
                a href="/projects/niri#niri-compatibility" {
                    strong { "Niri: check applications, dialogs and screen sharing →" }
                    span { "Separate X11 integration and portal setup from remote-control requirements with a version-aware acceptance checklist." }
                }
            }
            p class="small muted" { "Sources checked November 16, 2026. Suggested checks, not a tested desktop or remote-client configuration." }
        }
    }
}
