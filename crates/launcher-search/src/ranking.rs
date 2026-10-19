// Ported from Floe Search/Ranking.swift and SearchProviders/RootSearch.swift.
// Copyright (Floe) © 2026 René Jiménez. Licensed under GNU AGPLv3.
// Changes: value-only Rust data model, explicit settings/usage, stable sorting; no native providers.
use crate::fuzzy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    Application,
    Command,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub title: String,
    pub kind: ItemKind,
    #[serde(default)]
    pub keywords: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultRow {
    pub item: Item,
    pub section: Option<String>,
}
impl ResultRow {
    pub fn new(item: Item, section: Option<&str>) -> Self {
        Self {
            item,
            section: section.map(str::to_owned),
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct SearchContext {
    pub query: String,
    pub favorites: Vec<String>,
    pub hidden: HashSet<String>,
    pub aliases: HashMap<String, String>,
    pub frecency: HashMap<String, f64>,
}
impl SearchContext {
    fn usage(&self, id: &str) -> f64 {
        self.frecency.get(id).copied().unwrap_or(0.0)
    }
}
#[derive(Debug, Clone, Default)]
pub struct Contribution {
    pub ranked: Vec<Item>,
    pub search_only: Vec<Item>,
    pub pinned: Vec<ResultRow>,
    pub sectioned: Vec<ResultRow>,
    pub appended: Vec<ResultRow>,
}

pub fn frecency(count: u64, age: f64) -> f64 {
    let weight = if age < 3600.0 {
        4.0
    } else if age < 86400.0 {
        2.0
    } else if age < 604800.0 {
        1.0
    } else if age < 2592000.0 {
        0.5
    } else {
        0.25
    };
    count as f64 * weight
}
pub fn alias_score(query: &str, title: &str, alias: Option<&str>) -> Option<i32> {
    let title_score = fuzzy::score(query, title);
    let Some(alias) = alias.filter(|a| !a.is_empty()) else {
        return title_score;
    };
    let alias: String = fuzzy::lowercase(alias).nfc().collect();
    let query: String = fuzzy::lowercase(query).nfc().collect();
    if alias == query {
        Some(1000)
    } else if fuzzy::canonical_prefix(&alias, &query) {
        Some(title_score.unwrap_or(0).max(95))
    } else {
        title_score
    }
}
pub fn keyword_score(query: &str, keyword: &str) -> Option<i32> {
    fuzzy::tier_score(query, keyword).map(|s| s - 15)
}
pub fn menu_bar_score(query: &str, name: &str, owner: &str) -> Option<i32> {
    fuzzy::score(query, name)
        .into_iter()
        .chain(fuzzy::score(query, owner).map(|s| s - 10))
        .max()
}

pub fn browse(all: &[Item], search_only: &[Item], context: &SearchContext) -> Vec<ResultRow> {
    let known: Vec<_> = all.iter().chain(search_only).collect();
    let mut by_id = HashMap::new();
    for item in &known {
        by_id.entry(item.id.as_str()).or_insert(*item);
    }
    let favorites: Vec<_> = context
        .favorites
        .iter()
        .filter_map(|id| by_id.get(id.as_str()).copied())
        .collect();
    let favorite_ids: HashSet<_> = favorites.iter().map(|item| item.id.as_str()).collect();
    let mut suggestions: Vec<_> = known
        .iter()
        .copied()
        .filter(|item| !favorite_ids.contains(item.id.as_str()) && context.usage(&item.id) > 0.0)
        .collect();
    suggestions.sort_by(|a, b| context.usage(&b.id).total_cmp(&context.usage(&a.id)));
    suggestions.truncate(5);
    let shown: HashSet<_> = favorite_ids
        .into_iter()
        .chain(suggestions.iter().map(|item| item.id.as_str()))
        .collect();
    let mut result: Vec<_> = favorites
        .into_iter()
        .map(|item| ResultRow::new(item.clone(), Some("Favorites")))
        .collect();
    result.extend(
        suggestions
            .into_iter()
            .map(|item| ResultRow::new(item.clone(), Some("Suggestions"))),
    );
    for kind in [ItemKind::Command, ItemKind::Application] {
        let section = if kind == ItemKind::Command {
            "Commands"
        } else {
            "Applications"
        };
        result.extend(
            all.iter()
                .filter(|item| item.kind == kind && !shown.contains(item.id.as_str()))
                .map(|item| ResultRow::new(item.clone(), Some(section))),
        );
    }
    result
}

pub fn search(all: &[Item], context: &SearchContext, limit: usize) -> Vec<ResultRow> {
    let favorites: HashSet<_> = context.favorites.iter().map(String::as_str).collect();
    let mut scored: Vec<_> = all
        .iter()
        .filter_map(|item| {
            let title = alias_score(
                &context.query,
                &item.title,
                context.aliases.get(&item.id).map(String::as_str),
            );
            let score = title
                .into_iter()
                .chain(
                    item.keywords
                        .iter()
                        .filter_map(|k| keyword_score(&context.query, k)),
                )
                .max()?;
            let boost = (context.usage(&item.id) * 2.0).min(20.0)
                + if favorites.contains(item.id.as_str()) {
                    5.0
                } else {
                    0.0
                };
            Some((item, f64::from(score) + boost))
        })
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored
        .into_iter()
        .take(limit)
        .map(|(item, _)| ResultRow::new(item.clone(), None))
        .collect()
}

/// Same merge/filter boundaries as RootSearch; native provider implementations are not part of CP002.
pub fn merge(
    contributions: &[Contribution],
    context: &SearchContext,
    sources: &[ResultRow],
) -> Vec<ResultRow> {
    let pinned: Vec<_> = contributions
        .iter()
        .flat_map(|p| p.pinned.clone())
        .collect();
    let pinned_ids: HashSet<_> = pinned.iter().map(|r| r.item.id.as_str()).collect();
    let visible = |item: &&Item| !context.hidden.contains(&item.id);
    let ranked: Vec<_> = contributions
        .iter()
        .flat_map(|p| &p.ranked)
        .filter(visible)
        .cloned()
        .collect();
    let rows = if context.query.is_empty() {
        let only: Vec<_> = contributions
            .iter()
            .flat_map(|p| &p.search_only)
            .filter(visible)
            .cloned()
            .collect();
        browse(&ranked, &only, context)
    } else {
        let all: Vec<_> = contributions
            .iter()
            .flat_map(|p| p.ranked.iter().chain(&p.search_only))
            .filter(visible)
            .cloned()
            .collect();
        search(&all, context, 40)
    };
    let mut result: Vec<_> = contributions
        .iter()
        .flat_map(|p| p.sectioned.clone())
        .collect();
    result.extend(pinned.iter().cloned());
    result.extend(
        rows.into_iter()
            .filter(|r| !pinned_ids.contains(r.item.id.as_str())),
    );
    result.extend_from_slice(sources);
    result.extend(contributions.iter().flat_map(|p| p.appended.clone()));
    result
}
