use asaaan_personalization::{Operation, Store};
#[test]
fn command_usage_and_customizations_survive_restart_alongside_applications() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preferences.json");
    let mut store = Store::open(&path).unwrap();
    for id in ["app:notes", "command:list-lab/playground"] {
        store
            .execute(
                &Operation::RecordLaunch {
                    id: id.into(),
                    query: "planets".into(),
                },
                100,
            )
            .unwrap();
        store
            .execute(
                &Operation::Favorite {
                    id: id.into(),
                    enabled: true,
                },
                100,
            )
            .unwrap();
        store
            .execute(
                &Operation::Alias {
                    id: id.into(),
                    alias: "space".into(),
                },
                100,
            )
            .unwrap();
    }
    let saved = Store::open(&path).unwrap().snapshot();
    assert_eq!(
        saved.favorites,
        ["app:notes", "command:list-lab/playground"]
    );
    assert_eq!(saved.records["command:list-lab/playground"].count, 1);
    assert_eq!(saved.aliases["command:list-lab/playground"], "space");
    assert_eq!(saved.queries, ["planets"]);
    store
        .execute(
            &Operation::Hide {
                id: "command:list-lab/playground".into(),
                title: "Playground".into(),
            },
            200,
        )
        .unwrap();
    let mut store = Store::open(&path).unwrap();
    assert!(
        !store
            .snapshot()
            .favorites
            .contains(&"command:list-lab/playground".into())
    );
    store
        .execute(
            &Operation::Restore {
                id: "command:list-lab/playground".into(),
            },
            300,
        )
        .unwrap();
    let saved = Store::open(path).unwrap().snapshot();
    assert!(saved.hidden_results.is_empty());
    assert_eq!(saved.records["command:list-lab/playground"].count, 1);
    assert_eq!(saved.aliases["command:list-lab/playground"], "space");
    assert_eq!(saved.favorites, ["app:notes"]);
}
#[test]
fn malformed_command_identities_preserve_the_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preferences.json");
    let mut store = Store::open(&path).unwrap();
    store
        .execute(
            &Operation::RecordLaunch {
                id: "app:notes".into(),
                query: "".into(),
            },
            100,
        )
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    for id in [
        "command:",
        "command:x",
        "command:/x",
        "command:x/",
        "command:../x",
        "command:x/y/z",
        "command:x/y\n",
        "random:x/y",
    ] {
        assert!(
            store
                .execute(
                    &Operation::RecordLaunch {
                        id: id.into(),
                        query: "".into()
                    },
                    200
                )
                .is_err(),
            "{id:?}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}
