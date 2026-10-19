use asaaan_apps::{
    Application, Store, Target,
    search_command::{self, Options},
};
use asaaan_search::SearchContext;
use std::{collections::HashMap, sync::Arc};
fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| s.to_string()).collect()
}
fn catalog() -> Store {
    Store::new(Arc::new(|| {
        Ok((0..12)
            .map(|i| Application {
                id: format!("app:{i:02}"),
                name: format!("App {i:02}"),
                location: std::env::temp_dir().join(format!("app-{i}")),
                identity: Some(format!("id.{i}")),
                origin: None,
                keywords: if i == 4 {
                    vec!["keyword".into()]
                } else {
                    vec![]
                },
                target: Target::Bundle(std::env::temp_dir().join(format!("app-{i}"))),
            })
            .collect())
    }))
}
#[test]
fn search_options_are_explicit_and_isolated() {
    assert_eq!(
        search_command::parse(&args(&["--search", "cal"])).unwrap(),
        Some(Options {
            query: "cal".into()
        })
    );
    assert_eq!(
        search_command::parse(&args(&["--search="])).unwrap(),
        Some(Options { query: "".into() })
    );
    assert_eq!(
        search_command::parse(&args(&["--search=--literal"])).unwrap(),
        Some(Options {
            query: "--literal".into()
        })
    );
    assert!(
        search_command::parse(&args(&["--panel"]))
            .unwrap()
            .is_none()
    );
    assert!(
        search_command::parse(&args(&["--pick", "--query=--search"]))
            .unwrap()
            .is_none()
    );
    assert!(search_command::parse(&args(&["--help"])).unwrap().is_none());
}
#[test]
fn missing_repeated_and_conflicting_options_are_usage_errors() {
    for a in [
        vec!["--search"],
        vec!["--search", "--settings"],
        vec!["--search", "a", "--pick"],
        vec!["--search", "a", "--panel"],
        vec!["--settings", "--search", "a"],
        vec!["--search=a", "--search=b"],
        vec!["--search", "a", "extra"],
        vec!["--search", "a", "--quit"],
    ] {
        assert!(search_command::parse(&args(&a)).is_err(), "{a:?}");
    }
    assert!(search_command::parse(&["--search".into(), "x".repeat(4097)]).is_err());
    assert!(search_command::parse(&args(&["--search", "a\0b"])).is_err());
}
#[test]
fn prints_only_the_first_eight_in_ranked_order() {
    let text = search_command::render(&catalog(), "app", SearchContext::default()).unwrap();
    assert_eq!(
        text,
        (0..8)
            .map(|i| format!("App {i:02} (Application)\n"))
            .collect::<String>()
    );
}
#[test]
fn empty_query_preserves_sections_favorite_order_and_suggestions() {
    let ctx = SearchContext {
        favorites: vec!["app:11".into(), "app:05".into()],
        frecency: HashMap::from([("app:03".into(), 8.0)]),
        ..Default::default()
    };
    let text = search_command::render(&catalog(), "", ctx).unwrap();
    assert_eq!(
        text,
        "[Favorites] App 11 (Application)\n[Favorites] App 05 (Application)\n[Suggestions] App 03 (Application)\n[Applications] App 00 (Application)\n[Applications] App 01 (Application)\n[Applications] App 02 (Application)\n[Applications] App 04 (Application)\n[Applications] App 06 (Application)\n"
    );
}
#[test]
fn aliases_are_used_for_ranking_and_shown_on_title_matches_too() {
    let ctx = SearchContext {
        aliases: HashMap::from([("app:11".into(), "memo".into())]),
        ..Default::default()
    };
    assert_eq!(
        search_command::render(&catalog(), "memo", ctx.clone()).unwrap(),
        "App 11 (Application) (alias memo)\n"
    );
    assert_eq!(
        search_command::render(&catalog(), "11", ctx).unwrap(),
        "App 11 (Application) (alias memo)\n"
    );
}
#[test]
fn hidden_rows_are_filtered_before_the_display_limit() {
    let ctx = SearchContext {
        hidden: (0..5).map(|i| format!("app:{i:02}")).collect(),
        ..Default::default()
    };
    let text = search_command::render(&catalog(), "app", ctx).unwrap();
    assert_eq!(
        text,
        (5..12)
            .map(|i| format!("App {i:02} (Application)\n"))
            .collect::<String>()
    );
}
#[test]
fn usage_and_favorites_use_the_shared_ranking_context() {
    let ctx = SearchContext {
        favorites: vec!["app:11".into()],
        frecency: HashMap::from([("app:10".into(), 100.0)]),
        ..Default::default()
    };
    let text = search_command::render(&catalog(), "app", ctx).unwrap();
    assert!(text.starts_with("App 10 (Application)\nApp 11 (Application)\n"));
}
#[test]
fn keywords_and_no_matches_follow_the_existing_catalog() {
    assert_eq!(
        search_command::render(&catalog(), "keyword", SearchContext::default()).unwrap(),
        "App 04 (Application)\n"
    );
    assert_eq!(
        search_command::render(&catalog(), "unmatchedzzzz", SearchContext::default()).unwrap(),
        ""
    );
}
#[test]
fn unicode_is_preserved_and_terminal_controls_cannot_forge_rows() {
    let store = Store::new(Arc::new(|| {
        Ok(vec![Application {
            id: "app:unicode".into(),
            name: "日本語 ; literal\n\u{1b}[31m".into(),
            location: std::env::temp_dir(),
            identity: None,
            origin: None,
            keywords: vec![],
            target: Target::Bundle(std::env::temp_dir()),
        }])
    }));
    assert_eq!(
        search_command::render(&store, "", SearchContext::default()).unwrap(),
        "[Applications] 日本語 ; literal\\n\\u{1b}[31m (Application)\n"
    );
}
#[test]
fn scan_failures_propagate_instead_of_returning_partial_output() {
    let store = Store::new(Arc::new(|| Err("scan refused".into())));
    assert_eq!(
        search_command::render(&store, "", SearchContext::default()).unwrap_err(),
        "scan refused"
    );
}
