// Floe app aliases and hide-from-search equivalents, including CP005 migration.
use asaaan_personalization::{Operation, Snapshot, Store};
fn alias(id: &str, value: &str) -> Operation {
    Operation::Alias {
        id: id.into(),
        alias: value.into(),
    }
}
fn hide(id: &str, title: &str) -> Operation {
    Operation::Hide {
        id: id.into(),
        title: title.into(),
    }
}
fn item(id: &str, title: &str) -> asaaan_search::Item {
    asaaan_search::Item {
        id: id.into(),
        title: title.into(),
        kind: asaaan_search::ItemKind::Application,
        keywords: vec![],
    }
}
fn results(s: &Snapshot, query: &str) -> Vec<asaaan_search::ResultRow> {
    asaaan_search::merge(
        &[asaaan_search::Contribution {
            ranked: vec![item("app:safari", "Safari"), item("app:notes", "Notes")],
            ..Default::default()
        }],
        &s.context(query, 1000),
        &[],
    )
}
#[test]
fn checkpoint5_data_gains_empty_customization_without_losing_or_rewriting_values() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let bytes=br#"{"version":1,"favorites":["app:notes"],"records":{"app:notes":{"count":7,"lastUsed":123}},"queries":["notes"],"rememberSearches":false}"#;
    std::fs::write(&path, bytes).unwrap();
    let mut store = Store::open(&path).unwrap();
    let before = store.execute(&Operation::Read, 1000).unwrap();
    assert_eq!(before.version, 2);
    assert_eq!(before.favorites, ["app:notes"]);
    assert_eq!(before.records["app:notes"].count, 7);
    assert_eq!(before.queries, ["notes"]);
    assert!(!before.remember_searches);
    assert!(before.aliases.is_empty() && before.hidden_results.is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    store.execute(&alias("app:notes", "zz"), 1000).unwrap();
    let after = Store::open(&path).unwrap().snapshot();
    assert_eq!(after.version, 2);
    assert_eq!(after.records, before.records);
    assert_eq!(after.favorites, before.favorites);
    assert_eq!(after.queries, before.queries);
    assert!(!after.remember_searches);
    assert_eq!(after.aliases["app:notes"], "zz");
}
#[test]
fn exact_alias_and_prefix_find_the_application_while_title_still_works() {
    let mut state = Snapshot::default();
    state.apply(&alias("app:notes", "ZZ"), 0).unwrap();
    for query in ["zz", "z", "not"] {
        assert_eq!(results(&state, query)[0].item.id, "app:notes");
    }
    state.apply(&alias("app:notes", ""), 0).unwrap();
    assert!(results(&state, "zz").is_empty());
    assert_eq!(results(&state, "not")[0].item.id, "app:notes");
}
#[test]
fn alias_collisions_keep_distinct_apps_and_stable_ranking() {
    let mut state = Snapshot::default();
    state.apply(&alias("app:notes", "zz"), 0).unwrap();
    state.apply(&alias("app:safari", "zz"), 0).unwrap();
    assert_eq!(
        results(&state, "zz")
            .iter()
            .map(|r| r.item.id.as_str())
            .collect::<Vec<_>>(),
        ["app:safari", "app:notes"]
    );
}
#[test]
fn aliases_preserve_unicode_case_and_literal_spaces() {
    let mut state = Snapshot::default();
    state.apply(&alias("app:notes", "Café"), 0).unwrap();
    assert_eq!(results(&state, "CAFE\u{301}")[0].item.id, "app:notes");
    state.apply(&alias("app:notes", " zz "), 0).unwrap();
    assert_eq!(state.aliases["app:notes"], " zz ");
}
#[test]
fn hidden_results_leave_browse_search_favorites_and_suggestions() {
    let mut state = Snapshot::default();
    state
        .apply(
            &Operation::Favorite {
                id: "app:notes".into(),
                enabled: true,
            },
            0,
        )
        .unwrap();
    state
        .apply(
            &Operation::RecordLaunch {
                id: "app:notes".into(),
                query: "note".into(),
            },
            100,
        )
        .unwrap();
    state.apply(&alias("app:notes", "zz"), 0).unwrap();
    state.apply(&hide("app:notes", "Notes"), 0).unwrap();
    assert_eq!(state.hidden_results["app:notes"], "Notes");
    assert!(state.favorites.is_empty());
    for query in ["", "not", "zz"] {
        assert!(
            results(&state, query)
                .iter()
                .all(|row| row.item.id != "app:notes")
        );
    }
    assert!(
        state
            .apply(
                &Operation::Favorite {
                    id: "app:notes".into(),
                    enabled: true
                },
                0
            )
            .is_err()
    );
}
#[test]
fn showing_again_keeps_alias_usage_and_history_without_restoring_favorite() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut store = Store::open(&path).unwrap();
    for op in [
        Operation::Favorite {
            id: "app:notes".into(),
            enabled: true,
        },
        alias("app:notes", "zz"),
        Operation::RecordLaunch {
            id: "app:notes".into(),
            query: "zz".into(),
        },
        hide("app:notes", "Notes"),
    ] {
        store.execute(&op, 100).unwrap();
    }
    let mut next = Store::open(&path).unwrap();
    assert!(results(&next.snapshot(), "zz").is_empty());
    let restored = next
        .execute(
            &Operation::Restore {
                id: "app:notes".into(),
            },
            200,
        )
        .unwrap();
    assert!(restored.favorites.is_empty());
    assert_eq!(restored.queries, ["zz"]);
    assert_eq!(restored.records["app:notes"].count, 1);
    assert_eq!(results(&restored, "zz")[0].item.id, "app:notes");
    assert_eq!(restored, Store::open(&path).unwrap().snapshot());
}
#[test]
fn hidden_uninstalled_entries_can_be_restored_without_a_live_catalog() {
    let mut state = Snapshot::default();
    state
        .apply(&hide("app:removed", "Old Application"), 0)
        .unwrap();
    state
        .apply(
            &Operation::Restore {
                id: "app:removed".into(),
            },
            0,
        )
        .unwrap();
    assert!(state.hidden_results.is_empty());
    assert!(
        state
            .apply(
                &Operation::Restore {
                    id: "app:not-hidden".into()
                },
                0
            )
            .is_ok()
    );
}
#[test]
fn invalid_alias_and_hide_requests_do_not_publish_partial_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut store = Store::open(&path).unwrap();
    for op in [
        alias("bad", "x"),
        alias("app:notes", "bad\nvalue"),
        alias("app:notes", &"x".repeat(1025)),
        hide("app:notes", ""),
        Operation::Restore {
            id: "invalid".into(),
        },
    ] {
        assert!(store.execute(&op, 0).is_err());
        assert_eq!(store.snapshot(), Snapshot::default());
    }
    assert!(!path.exists());
}

#[test]
fn ranking_reads_aliases_and_frecency_from_persisted_context() {
    let mut state = Snapshot::default();
    let apps = vec![item("app:safari", "Safari"), item("app:sandbox", "Sandbox")];
    let search = |state: &Snapshot, query: &str| {
        asaaan_search::merge(
            &[asaaan_search::Contribution {
                ranked: apps.clone(),
                ..Default::default()
            }],
            &state.context(query, 1000),
            &[],
        )
    };
    assert_eq!(search(&state, "sa")[0].item.id, "app:safari");
    for _ in 0..3 {
        state
            .apply(
                &Operation::RecordLaunch {
                    id: "app:sandbox".into(),
                    query: "".into(),
                },
                1000,
            )
            .unwrap();
    }
    assert_eq!(search(&state, "sa")[0].item.id, "app:sandbox");
    state.apply(&alias("app:sandbox", "zz"), 1000).unwrap();
    assert_eq!(
        search(&state, "zz")
            .iter()
            .map(|r| r.item.id.as_str())
            .collect::<Vec<_>>(),
        ["app:sandbox"]
    );
}
