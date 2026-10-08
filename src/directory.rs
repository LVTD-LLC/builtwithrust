//! Browse the published catalog without changing SQL or the cached default hot path.
use crate::db::Project;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Browse {
    pub q: String,
    pub category: String,
    pub stack: String,
    pub license: String,
    pub source: String,
    pub verified: String,
    pub stars: String,
    pub sort: String,
}

impl Browse {
    pub fn normalize(&mut self) {
        for value in [&mut self.q, &mut self.category, &mut self.stack, &mut self.license] {
            *value = value.trim().chars().take(100).collect();
        }
        for (value, allowed) in [
            (&mut self.source, &["available", "unlisted"][..]),
            (&mut self.verified, &["1"][..]),
            (&mut self.stars, &["under-1000", "1000", "10000"][..]),
            (&mut self.sort, &["stars", "newest", "name", "gems"][..]),
        ] {
            if !allowed.contains(&value.as_str()) {
                value.clear();
            }
        }
    }

    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }

    /// Input is already in featured-first order; explicit sorts deliberately override it.
    pub fn apply(&self, projects: &mut Vec<Project>) {
        let query = self.q.to_lowercase();
        projects.retain(|p| {
            let has_source = p.repo_url.as_deref().is_some_and(|s| !s.trim().is_empty());
            (query.is_empty()
                || [&p.name, &p.tagline, &p.description, &p.stack].iter().any(|s| s.to_lowercase().contains(&query)))
                && (self.category.is_empty() || p.category_slug.as_deref() == Some(&self.category))
                && (self.stack.is_empty() || p.stack_list().iter().any(|s| s.trim().eq_ignore_ascii_case(&self.stack)))
                && (self.license.is_empty()
                    || p.license.as_deref().is_some_and(|s| s.trim().eq_ignore_ascii_case(&self.license)))
                && (self.verified.is_empty() || p.is_verified())
                && match self.source.as_str() {
                    "available" => has_source,
                    "unlisted" => !has_source,
                    _ => true,
                }
                && match self.stars.as_str() {
                    "under-1000" => p.stars.is_some_and(|n| (0..1000).contains(&n)),
                    "1000" => p.stars.is_some_and(|n| n >= 1000),
                    "10000" => p.stars.is_some_and(|n| n >= 10000),
                    _ => true,
                }
                && (self.sort != "gems" || p.stars.is_some_and(|n| (0..1000).contains(&n)))
        });
        if !self.sort.is_empty() {
            projects.sort_by_cached_key(|p| (p.name.to_lowercase(), p.slug.clone()));
            // Stable sorts retain alphabetical tie breaks. Unknown stars always go last.
            match self.sort.as_str() {
                "stars" => projects.sort_by_key(|p| std::cmp::Reverse(p.stars)),
                "newest" => projects.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
                "gems" => projects.sort_by_key(|p| p.stars),
                _ => {}
            }
        }
    }
}

pub struct Facets {
    pub stacks: Vec<String>,
    pub licenses: Vec<String>,
}

impl Facets {
    pub fn from_projects(projects: &[Project]) -> Self {
        let stacks = projects
            .iter()
            .flat_map(Project::stack_list)
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let licenses = projects
            .iter()
            .filter_map(|p| p.license.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Self { stacks, licenses }
    }
}
