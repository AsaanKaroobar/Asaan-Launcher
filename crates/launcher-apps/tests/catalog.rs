// Floe AppScan/CatalogLoader application equivalents, © 2026 René Jiménez. GNU AGPLv3.
#![allow(non_snake_case)]
use asaaan_apps::{Application, Store, Target, bundles, distinct};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
fn app(name: &str, path: &str, identity: Option<&str>) -> Application {
    Application {
        id: format!("app:{path}"),
        name: name.into(),
        location: path.into(),
        identity: identity.map(str::to_owned),
        origin: None,
        keywords: vec![],
        target: Target::Bundle(path.into()),
    }
}
struct Temp(PathBuf);
impl Temp {
    fn new(label: &str) -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "asaaan-{label}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn folders(&self, paths: &[&str]) {
        for path in paths {
            std::fs::create_dir_all(self.0.join(path)).unwrap();
        }
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[test]
fn aWebAppABrowserInstalledIsFoundInItsFolder() {
    let t = Temp::new("bundles");
    t.folders(&[
        "Notes.app",
        "Helium Apps.localized/PairDrop.app",
        "Utilities/Terminal.app",
        "Suite/Deeper/Buried.app",
        ".hidden/Secret.app",
        "Notes.app/Contents/Helpers/Helper.app",
    ]);
    let scanned = bundles::scan(std::slice::from_ref(&t.0));
    assert_eq!(
        scanned.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
        ["Notes", "PairDrop", "Terminal"]
    );
    assert_eq!(bundles::folders(std::slice::from_ref(&t.0)).len(), 4);
}
#[test]
fn twoAppsOfOneNameAreBothKeptAndTheSameAppTwiceIsOne() {
    let apps = distinct(vec![
        app(
            "Notes",
            "/System/Applications/Notes.app",
            Some("apple.notes"),
        ),
        app("Safari", "/Applications/Safari.app", Some("apple.safari")),
        app("Chess", "/System/Applications/Chess.app", None),
        app(
            "Notes",
            "/Users/me/Applications/Helium Apps.localized/Notes.app",
            Some("helium.notes"),
        ),
        app(
            "Safari",
            "/System/Cryptexes/Applications/Safari.app",
            Some("apple.safari"),
        ),
    ]);
    assert_eq!(
        apps.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
        ["Notes", "Notes", "Safari", "Chess"]
    );
    assert_eq!(
        apps.iter().map(|a| a.origin.as_deref()).collect::<Vec<_>>(),
        [Some("Applications"), Some("Helium Apps"), None, None]
    );
}
#[test]
fn twoAppsShownUnderOneNameInOneFolderAreToldApartByTheirFiles() {
    let apps = distinct(vec![
        app("Siri", "/Applications/Siri.app", Some("one")),
        app("Siri", "/Applications/Siri AI.app", Some("two")),
    ]);
    assert_eq!(
        apps.iter().map(|a| a.origin.as_deref()).collect::<Vec<_>>(),
        [Some("Siri"), Some("Siri AI")]
    );
}
#[test]
fn appsWithNoIdentifierAreToldApartByWhereTheyAre() {
    let a = app("Tool", "/Applications/Tool.app", None);
    let b = app("Tool", "/Applications/Suite/Tool.app", None);
    assert_eq!(distinct(vec![a.clone(), b]).len(), 2);
    assert_eq!(distinct(vec![a.clone(), a]).len(), 1);
}
#[test]
fn aFolderMadeLaterIsWatchedWithoutARelaunch() {
    let t = Temp::new("refresh");
    t.folders(&["Notes.app"]);
    assert_eq!(bundles::scan(std::slice::from_ref(&t.0)).len(), 1);
    t.folders(&["Helium Apps.localized/PairDrop.app"]);
    assert_eq!(bundles::scan(std::slice::from_ref(&t.0)).len(), 2);
    std::fs::remove_dir_all(t.0.join("Notes.app")).unwrap();
    assert_eq!(
        bundles::scan(std::slice::from_ref(&t.0))[0].name,
        "PairDrop"
    );
}
#[test]
fn aFolderThatIsNotThereIsNothingToScan() {
    let t = Temp::new("missing");
    let root = t.0.join("missing");
    assert_eq!(
        bundles::folders(std::slice::from_ref(&root)),
        vec![root.clone()]
    );
    assert!(bundles::scan(&[root]).is_empty());
}
#[test]
fn bundleMetadataAndMiddleDotAppNamesArePreserved() {
    let t = Temp::new("metadata");
    t.folders(&["Foo.app Maker.app/Contents"]);
    let mut dict = plist::Dictionary::new();
    dict.insert(
        "CFBundleDisplayName".into(),
        plist::Value::String("Localized Display".into()),
    );
    dict.insert(
        "CFBundleIdentifier".into(),
        plist::Value::String("test.bundle".into()),
    );
    plist::Value::Dictionary(dict)
        .to_file_binary(t.0.join("Foo.app Maker.app/Contents/Info.plist"))
        .unwrap();
    let a = bundles::scan(std::slice::from_ref(&t.0));
    assert_eq!(a[0].name, "Localized Display");
    assert_eq!(a[0].identity.as_deref(), Some("test.bundle"));
    std::fs::remove_file(t.0.join("Foo.app Maker.app/Contents/Info.plist")).unwrap();
    assert_eq!(
        bundles::scan(std::slice::from_ref(&t.0))[0].name,
        "Foo.app Maker"
    );
}
#[test]
fn constructionStartsNoScans() {
    let count = Arc::new(AtomicUsize::new(0));
    let clone = count.clone();
    let _store = Store::new(Arc::new(move || {
        clone.fetch_add(1, Ordering::SeqCst);
        Ok(vec![])
    }));
    assert_eq!(count.load(Ordering::SeqCst), 0);
}
#[test]
fn scanAppsReturnsEachAppOnceSortedAndTellsApartTheOnesThatShareAName() {
    let store = Store::new(Arc::new(|| {
        Ok(vec![
            app("Zebra", "/apps/Zebra.app", None),
            app("Alpha", "/apps/Alpha.app", None),
            app("Alpha", "/apps/Alpha.app", None),
        ])
    }));
    let result = store.search("", false).unwrap();
    assert_eq!(result.total, 2);
    assert_eq!(
        result
            .rows
            .iter()
            .map(|r| r.item.title.as_str())
            .collect::<Vec<_>>(),
        ["Alpha", "Zebra"]
    );
}
#[test]
fn aSnapshotCarriesItsCatalogUntouched() {
    let source = vec![app("Safari", "/apps/Safari.app", None)];
    let copy = source.clone();
    let store = Store::new(Arc::new(move || Ok(copy.clone())));
    store.search("", false).unwrap();
    assert_eq!(store.get(&source[0].id).unwrap(), source[0]);
}
#[test]
fn anEmptyCatalogStillCompletesReadiness() {
    let store = Store::default();
    let isolated = Store::new(Arc::new(|| Ok(vec![])));
    let reply = isolated.search("", false).unwrap();
    assert_eq!(reply.generation, 1);
    assert!(reply.rows.is_empty());
    drop(store);
}
#[test]
fn readinessCallersShareOneLoad() {
    let count = Arc::new(AtomicUsize::new(0));
    let cloned = count.clone();
    let store = Store::new(Arc::new(move || {
        cloned.fetch_add(1, Ordering::SeqCst);
        Ok(vec![])
    }));
    store.search("a", false).unwrap();
    store.search("b", false).unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
#[test]
fn aRescanKeepsThePreviousResultsVisible() {
    let n = AtomicUsize::new(0);
    let store = Store::new(Arc::new(move || {
        if n.fetch_add(1, Ordering::SeqCst) == 0 {
            Ok(vec![app("Notes", "/apps/Notes.app", None)])
        } else {
            Err("scan failed".into())
        }
    }));
    store.search("", false).unwrap();
    let refreshed = store.search("", true).unwrap();
    assert_eq!(refreshed.rows[0].item.title, "Notes");
    assert!(refreshed.warning.unwrap().contains("scan failed"));
}
#[test]
fn olderAppsResultsCannotOverwriteNewerApps() {
    let count = AtomicUsize::new(0);
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let gate = Arc::new(std::sync::Barrier::new(2));
    let gate_clone = gate.clone();
    let store = Arc::new(Store::new(Arc::new(move || {
        let n = count.fetch_add(1, Ordering::SeqCst);
        if n == 0 {
            started_tx.send(()).unwrap();
            gate_clone.wait();
        }
        Ok(vec![app(
            if n == 0 { "Old" } else { "New" },
            if n == 0 {
                "/apps/Old.app"
            } else {
                "/apps/New.app"
            },
            None,
        )])
    })));
    let a = store.clone();
    let first = std::thread::spawn(move || a.search("", true));
    started_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let b = store.clone();
    let next = std::thread::spawn(move || b.search("", true));
    gate.wait();
    first.join().unwrap().unwrap();
    next.join().unwrap().unwrap();
    assert!(store.get("app:/apps/Old.app").is_err());
    assert_eq!(
        store.search("New", false).unwrap().rows[0].item.title,
        "New"
    );
}
#[test]
fn removedAndUnknownIdsCannotLaunch() {
    let store = Store::new(Arc::new(|| Ok(vec![])));
    assert!(store.get("/bin/sh").is_err());
    assert!(store.get("app:../../other").is_err());
}
#[test]
fn windowsCatalogParsesSingletonArrayAndEmptyWithoutDroppingSameNameApps() {
    use asaaan_apps::windows::parse;
    assert!(parse("null").unwrap().is_empty());
    assert_eq!(parse(r#"{"Name":"Notes","AppID":"one"}"#).unwrap().len(), 1);
    let apps=parse(r#"[{"Name":"Notes","AppID":"one"},{"Name":"Notes","AppID":"two"},{"Name":"Duplicate","AppID":"ONE"}]"#).unwrap();
    assert_eq!(apps.len(), 2);
    assert!(parse("invalid").is_err());
    assert!(
        parse(r#"[{"Name":"Bad","AppID":"x\n"}]"#)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn queryUsesPreviousSnapshotWhileARefreshIsSuspended() {
    let count = AtomicUsize::new(0);
    let (tx, rx) = std::sync::mpsc::channel();
    let gate = Arc::new(std::sync::Barrier::new(2));
    let worker_gate = gate.clone();
    let store = Arc::new(Store::new(Arc::new(move || {
        if count.fetch_add(1, Ordering::SeqCst) > 0 {
            tx.send(()).unwrap();
            worker_gate.wait();
        }
        Ok(vec![app("Notes", "/apps/Notes.app", None)])
    })));
    store.search("", false).unwrap();
    let copied = store.clone();
    let refresh = std::thread::spawn(move || copied.search("", true));
    rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    let reply = store.search("notes", false).unwrap();
    assert_eq!(reply.rows[0].item.title, "Notes");
    gate.wait();
    refresh.join().unwrap().unwrap();
}
#[test]
fn theWorkerIsACatalogScanner() {
    let store = Store::new(Arc::new(|| Ok(vec![])));
    assert_eq!(store.search("", false).unwrap().generation, 1);
}

#[test]
fn personalized_context_changes_the_real_catalog_order_without_losing_apps() {
    let store = asaaan_apps::Store::new(std::sync::Arc::new(|| {
        Ok(vec![
            asaaan_apps::Application {
                id: "app:a".into(),
                name: "Alpha".into(),
                location: "/a".into(),
                identity: Some("a".into()),
                origin: None,
                keywords: vec![],
                target: asaaan_apps::Target::Bundle("/a".into()),
            },
            asaaan_apps::Application {
                id: "app:z".into(),
                name: "Zulu".into(),
                location: "/z".into(),
                identity: Some("z".into()),
                origin: None,
                keywords: vec![],
                target: asaaan_apps::Target::Bundle("/z".into()),
            },
        ])
    }));
    let context = asaaan_search::SearchContext {
        favorites: vec!["app:z".into()],
        frecency: std::collections::HashMap::from([("app:a".into(), 4.0)]),
        ..Default::default()
    };
    let rows = store.search_with_context("", false, context).unwrap().rows;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].item.id, "app:z");
    assert_eq!(rows[0].section.as_deref(), Some("Favorites"));
    assert_eq!(rows[1].section.as_deref(), Some("Suggestions"));
}

#[test]
fn bundled_commands_share_ranking_customizations_but_have_no_application_metadata() {
    use asaaan_search::{Item, ItemKind, SearchContext};
    let store = Store::new(Arc::new(|| {
        Ok(vec![app("Notes", "/Applications/Notes.app", None)])
    }));
    let command = Item {
        id: "command:list-lab/planets".into(),
        title: "Browse Planets".into(),
        kind: ItemKind::Command,
        keywords: vec!["space".into()],
    };
    let initial = store
        .search_with_extra(
            "",
            false,
            SearchContext::default(),
            std::slice::from_ref(&command),
        )
        .unwrap();
    assert_eq!(initial.total, 1);
    assert_eq!(
        initial
            .rows
            .iter()
            .map(|r| r.item.id.as_str())
            .collect::<Vec<_>>(),
        [command.id.as_str(), "app:/Applications/Notes.app"]
    );
    assert!(initial.rows[0].actions.is_empty());
    assert!(initial.rows[0].origin.is_none());
    assert!(store.get(&command.id).is_err());
    let mut context = SearchContext::default();
    context.aliases.insert(command.id.clone(), "solar".into());
    let aliased = store
        .search_with_extra(
            "solar",
            false,
            context.clone(),
            std::slice::from_ref(&command),
        )
        .unwrap();
    assert_eq!(aliased.rows[0].item.id, command.id);
    context.hidden.insert(command.id.clone());
    assert!(
        store
            .search_with_extra("solar", false, context, std::slice::from_ref(&command))
            .unwrap()
            .rows
            .is_empty()
    );
    // Existing clients, including --search, keep the application-only catalog.
    assert_eq!(store.search("", false).unwrap().rows.len(), 1);
}
