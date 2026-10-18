use asaaan_personalization::{Operation, Store, Theme};
#[test]
fn old_files_default_to_floe_without_rewriting_and_theme_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preferences.json");
    let old=br#"{"version":2,"favorites":["app:notes"],"queries":["kept"],"aliases":{"app:notes":"nt"}}"#;
    std::fs::write(&path, old).unwrap();
    let mut store = Store::open(&path).unwrap();
    assert_eq!(store.snapshot().theme, Theme::Floe);
    assert_eq!(std::fs::read(&path).unwrap(), old);
    store
        .execute(
            &Operation::Theme {
                theme: Theme::Legacy,
            },
            0,
        )
        .unwrap();
    let saved = Store::open(&path).unwrap().snapshot();
    assert_eq!(saved.theme, Theme::Legacy);
    assert_eq!(saved.favorites, ["app:notes"]);
    assert_eq!(saved.queries, ["kept"]);
    assert_eq!(saved.aliases["app:notes"], "nt");
    store
        .execute(&Operation::Theme { theme: Theme::Floe }, 0)
        .unwrap();
    assert_eq!(Store::open(&path).unwrap().snapshot().theme, Theme::Floe);
}
#[test]
fn invalid_theme_is_rejected_without_replacing_the_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preferences.json");
    let bytes = br#"{"version":2,"theme":"unknown"}"#;
    std::fs::write(&path, bytes).unwrap();
    assert!(Store::open(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert!(serde_json::from_str::<Operation>(r#"{"kind":"theme","theme":"unknown"}"#).is_err());
}
