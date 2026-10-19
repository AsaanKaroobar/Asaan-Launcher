// Equivalent pure merge assertions from Floe SearchProviderTests.swift.
// Copyright (Floe) © 2026 René Jiménez. Licensed under GNU AGPLv3.
#![allow(non_snake_case)]
use asaaan_search::*;
fn item(id: &str, title: &str) -> Item {
    Item {
        id: id.into(),
        title: title.into(),
        kind: ItemKind::Application,
        keywords: vec![],
    }
}
fn row(id: &str, section: Option<&str>) -> ResultRow {
    ResultRow::new(item(id, id), section)
}
fn ids(rows: &[ResultRow]) -> Vec<&str> {
    rows.iter().map(|r| r.item.id.as_str()).collect()
}
#[test]
fn theMergePutsSectionedRowsFirstThenPinnedThenRankedThenAppended() {
    let p = Contribution {
        sectioned: vec![row("event:Standup", Some("Today"))],
        pinned: vec![row("calculator", None)],
        ranked: vec![item("app:Safari", "Safari")],
        appended: vec![row("files-for:saf", None)],
        ..Default::default()
    };
    let c = SearchContext {
        query: "saf".into(),
        ..Default::default()
    };
    assert_eq!(
        ids(&merge(&[p], &c, &[])),
        ["event:Standup", "calculator", "app:Safari", "files-for:saf"]
    );
}
#[test]
fn rowsOfOneKindKeepTheOrderOfTheirProviders() {
    let first = Contribution {
        pinned: vec![row("note:new", None)],
        appended: vec![row("files-for:one", None)],
        ..Default::default()
    };
    let second = Contribution {
        pinned: vec![row("script:deploy.sh", None)],
        appended: vec![row("files-for:two", None)],
        ..Default::default()
    };
    assert_eq!(
        ids(&merge(&[first, second], &Default::default(), &[])),
        [
            "note:new",
            "script:deploy.sh",
            "files-for:one",
            "files-for:two"
        ]
    );
}
#[test]
fn aPinnedRowTakesThePlaceOfTheRankedRowWithItsId() {
    let p = Contribution {
        pinned: vec![row("deploy", None)],
        ranked: vec![item("deploy", "Deploy"), item("app:Deployer", "Deployer")],
        ..Default::default()
    };
    assert_eq!(
        ids(&merge(
            &[p],
            &SearchContext {
                query: "dep".into(),
                ..Default::default()
            },
            &[]
        )),
        ["deploy", "app:Deployer"]
    );
}
#[test]
fn withoutAQuerySearchOnlyRowsShowOnlyAsFavoritesOrSuggestions() {
    let p = Contribution {
        ranked: vec![item("safari", "Safari")],
        search_only: vec![item("displays", "Displays"), item("sound", "Sound")],
        ..Default::default()
    };
    let c = SearchContext {
        favorites: vec!["displays".into()],
        ..Default::default()
    };
    let rows = merge(std::slice::from_ref(&p), &c, &[]);
    assert_eq!(ids(&rows), ["displays", "safari"]);
    assert_eq!(
        rows.iter()
            .map(|r| r.section.as_deref())
            .collect::<Vec<_>>(),
        [Some("Favorites"), Some("Applications")]
    );
    assert_eq!(
        ids(&merge(
            &[p],
            &SearchContext {
                query: "sou".into(),
                ..Default::default()
            },
            &[]
        )),
        ["sound"]
    );
}
#[test]
fn hidden_filter_is_applied_to_ranked_candidates_only_and_source_position_is_preserved() {
    let p = Contribution {
        sectioned: vec![row("section", None)],
        pinned: vec![row("pin", None)],
        ranked: vec![item("hide", "Safari"), item("keep", "Safari Two")],
        search_only: vec![item("hide-only", "Safari Three")],
        appended: vec![row("last", None)],
    };
    let c = SearchContext {
        query: "saf".into(),
        hidden: ["hide", "hide-only", "pin"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        ..Default::default()
    };
    assert_eq!(
        ids(&merge(&[p], &c, &[row("source", Some("Files"))])),
        ["section", "pin", "keep", "source", "last"]
    );
}
#[test]
fn per_provider_ranked_and_search_only_interleaving_breaks_equal_scores_in_source_order() {
    let p = Contribution {
        ranked: vec![item("one", "Same")],
        search_only: vec![item("two", "Same")],
        ..Default::default()
    };
    let q = Contribution {
        ranked: vec![item("three", "Same")],
        ..Default::default()
    };
    assert_eq!(
        ids(&merge(
            &[p, q],
            &SearchContext {
                query: "same".into(),
                ..Default::default()
            },
            &[]
        )),
        ["one", "two", "three"]
    );
}
