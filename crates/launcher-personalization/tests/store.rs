// Floe UsageStore/Settings/ItemAction equivalents, plus native persistence failures.
use asaaan_personalization::{Operation, Snapshot, Store};
fn launch(id: &str, query: &str) -> Operation {
    Operation::RecordLaunch {
        id: id.into(),
        query: query.into(),
    }
}
fn favorite(id: &str, enabled: bool) -> Operation {
    Operation::Favorite {
        id: id.into(),
        enabled,
    }
}
#[test]
fn never_used_has_no_frecency() {
    assert!(Snapshot::default().context("", 100).frecency.is_empty());
}
#[test]
fn each_use_counts_immediately() {
    let mut s = Snapshot::default();
    s.apply(&launch("app:notes", ""), 100_000).unwrap();
    s.apply(&launch("app:notes", ""), 100_000).unwrap();
    assert_eq!(s.records["app:notes"].count, 2);
    assert_eq!(s.context("", 100_000).frecency["app:notes"], 8.0);
}
#[test]
fn frecency_fades_with_the_reference_clock_boundaries() {
    let mut s = Snapshot::default();
    s.apply(&launch("app:notes", ""), 100_000).unwrap();
    for (age, expected) in [
        (0, 4.0),
        (3_600_000, 2.0),
        (86_400_000, 1.0),
        (604_800_000, 0.5),
        (2_592_000_000, 0.25),
    ] {
        assert_eq!(s.context("", 100_000 + age).frecency["app:notes"], expected);
    }
    assert_eq!(s.context("", 0).frecency["app:notes"], 4.0);
}
#[test]
fn persisted_uses_favorites_and_queries_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut store = Store::open(&path).unwrap();
    store.execute(&launch("app:notes", " notes "), 100).unwrap();
    store.execute(&launch("app:notes", ""), 200).unwrap();
    store.execute(&favorite("app:notes", true), 200).unwrap();
    let next = Store::open(&path).unwrap().snapshot();
    assert_eq!(next.records["app:notes"].count, 2);
    assert_eq!(next.records["app:notes"].last_used, 200);
    assert_eq!(next.queries, ["notes"]);
    assert_eq!(next.favorites, ["app:notes"]);
}
#[test]
fn acknowledged_writes_need_no_timer_or_foreground_flush() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut store = Store::open(&path).unwrap();
    for id in ["app:one", "app:two", "app:one"] {
        store.execute(&launch(id, ""), 100).unwrap();
    }
    let disk = Store::open(&path).unwrap().snapshot();
    assert_eq!(disk.records["app:one"].count, 2);
    assert_eq!(disk.records["app:two"].count, 1);
    assert_eq!(disk, store.snapshot());
}
#[test]
fn queries_are_trimmed_unique_bounded_and_clearable() {
    let mut s = Snapshot::default();
    for q in ["  safari ", "12 * 4", "   ", "safari"] {
        s.apply(&launch("app:notes", q), 100).unwrap();
    }
    assert_eq!(s.queries, ["12 * 4", "safari"]);
    for i in 0..50 {
        s.apply(&launch("app:notes", &format!("search {i}")), 100)
            .unwrap();
    }
    assert_eq!(s.queries.len(), 50);
    assert_eq!(s.queries.first().unwrap(), "search 0");
    s.apply(&Operation::ClearQueries, 100).unwrap();
    assert!(s.queries.is_empty());
    assert_eq!(s.records["app:notes"].count, 54);
}
#[test]
fn opting_out_preserves_old_queries_and_keeps_usage_separate() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut store = Store::open(&path).unwrap();
    store.execute(&launch("app:notes", "old"), 100).unwrap();
    store
        .execute(&Operation::RememberSearches { enabled: false }, 100)
        .unwrap();
    store
        .execute(&launch("app:notes", "private query"), 200)
        .unwrap();
    let disk = Store::open(&path).unwrap().snapshot();
    assert!(!disk.remember_searches);
    assert_eq!(disk.queries, ["old"]);
    assert_eq!(disk.records["app:notes"].count, 2);
    store.execute(&Operation::ClearQueries, 200).unwrap();
    assert!(Store::open(&path).unwrap().snapshot().queries.is_empty());
}
#[test]
fn favorites_are_ordered_idempotent_and_readding_goes_last() {
    let mut s = Snapshot::default();
    for (id, on) in [
        ("app:a", true),
        ("app:b", true),
        ("app:a", true),
        ("app:a", false),
        ("app:a", true),
    ] {
        s.apply(&favorite(id, on), 0).unwrap();
    }
    assert_eq!(s.favorites, ["app:b", "app:a"]);
}
#[test]
fn missing_catalog_ids_are_retained_without_inventing_rows() {
    let mut s = Snapshot::default();
    s.apply(&favorite("app:temporarily-uninstalled", true), 0)
        .unwrap();
    let c = s.context("", 0);
    assert!(asaaan_search::merge(&[], &c, &[]).is_empty());
    assert_eq!(s.favorites, ["app:temporarily-uninstalled"]);
}
#[test]
fn malformed_and_future_files_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    for bytes in [
        b"{broken".to_vec(),
        br#"{"version":99}"#.to_vec(),
        br#"{"version":1,"favorites":["bad"]}"#.to_vec(),
    ] {
        std::fs::write(&path, &bytes).unwrap();
        assert!(Store::open(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}
#[test]
fn failed_write_does_not_publish_uncommitted_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let backup = dir.path().join("backup.json");
    let mut store = Store::open(&path).unwrap();
    store.execute(&launch("app:one", "one"), 100).unwrap();
    let before = store.snapshot();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::rename(&path, &backup).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert!(store.execute(&favorite("app:one", true), 100).is_err());
    assert_eq!(store.snapshot(), before);
    assert_eq!(std::fs::read(&backup).unwrap(), bytes);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
}
#[test]
fn invalid_mutations_are_rejected_without_changing_the_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut store = Store::open(&path).unwrap();
    for op in [
        favorite("bad", true),
        launch("app:bad\n", "oops"),
        launch("app:one", &"x".repeat(4097)),
    ] {
        assert!(store.execute(&op, 0).is_err());
        assert_eq!(store.snapshot(), Snapshot::default());
    }
    assert!(!path.exists());
}
#[cfg(unix)]
#[test]
fn persisted_file_has_owner_only_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut store = Store::open(&path).unwrap();
    store.execute(&favorite("app:one", true), 100).unwrap();
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn oversized_files_are_bounded_and_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let bytes = vec![b'x'; 2 * 1024 * 1024 + 1];
    std::fs::write(&path, &bytes).unwrap();
    assert!(Store::open(&path).err().unwrap().contains("too large"));
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
#[test]
fn corrupt_value_diagnostics_do_not_echo_private_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    std::fs::write(
        &path,
        br#"{"version":1,"records":{"app:one":{"count":"sensitive-query-text","lastUsed":0}}}"#,
    )
    .unwrap();
    let error = Store::open(path).err().unwrap();
    assert!(!error.contains("sensitive-query-text"));
    assert!(error.contains("existing file preserved"));
}
