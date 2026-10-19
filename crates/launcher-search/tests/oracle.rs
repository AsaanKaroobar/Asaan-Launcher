use asaaan_search::{fuzzy, ranking::*};
use serde::Deserialize;
use std::collections::HashMap;
#[derive(Deserialize)]
struct Suite {
    rankings: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    input: Input,
    expected: Vec<Expected>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawItem {
    id: String,
    title: String,
    is_app: bool,
    keywords: Vec<String>,
}
impl From<RawItem> for Item {
    fn from(i: RawItem) -> Self {
        Self {
            id: i.id,
            title: i.title,
            kind: if i.is_app {
                ItemKind::Application
            } else {
                ItemKind::Command
            },
            keywords: i.keywords,
        }
    }
}
#[derive(Deserialize)]
struct Expected {
    item: RawItem,
    section: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Input {
    name: String,
    mode: String,
    query: String,
    items: Vec<RawItem>,
    search_only: Vec<RawItem>,
    favorites: Vec<String>,
    aliases: HashMap<String, String>,
    usage: HashMap<String, f64>,
    limit: usize,
}
#[test]
fn original_swift_ranking_snapshots_match() {
    let suite: Suite =
        serde_json::from_str(include_str!("../../../tests/fixtures/floe-search.json")).unwrap();
    assert!(suite.rankings.len() >= 20);
    for case in suite.rankings {
        let input = case.input;
        let all: Vec<_> = input.items.into_iter().map(Item::from).collect();
        let only: Vec<_> = input.search_only.into_iter().map(Item::from).collect();
        let context = SearchContext {
            query: input.query,
            favorites: input.favorites,
            aliases: input.aliases,
            frecency: input.usage,
            ..Default::default()
        };
        let actual = if input.mode == "browse" {
            browse(&all, &only, &context)
        } else {
            search(&all, &context, input.limit)
        };
        let expected: Vec<_> = case
            .expected
            .into_iter()
            .map(|r| ResultRow {
                item: r.item.into(),
                section: r.section,
            })
            .collect();
        assert_eq!(actual, expected, "{}", input.name);
    }
}
#[test]
fn canonical_equivalence_preserves_original_character_positions() {
    assert_eq!(fuzzy::match_text("é", "Cafe\u{301}").unwrap().matched, [3]);
    assert_eq!(fuzzy::match_text("e\u{301}", "Café").unwrap().matched, [3]);
    assert_eq!(fuzzy::match_text("b", "a👨‍👩‍👧‍👦b").unwrap().matched, [2]);
    assert_eq!(fuzzy::score("👩", "👩‍💻"), None);
}
