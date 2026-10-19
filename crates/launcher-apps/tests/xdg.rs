#![cfg(target_os = "linux")]
use std::{
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "asaaan xdg {}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(root.join("user/applications")).unwrap();
        std::fs::create_dir_all(root.join("system/applications")).unwrap();
        Self { root }
    }
    fn entry(&self, scope: &str, id: &str, body: &str) {
        let p = self.root.join(scope).join("applications").join(id);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(
            p,
            format!("[Desktop Entry]\nType=Application\nExec=/bin/true\n{body}\n"),
        )
        .unwrap();
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_app-catalog-fixture"))
            .args(args)
            .env("XDG_DATA_HOME", self.root.join("user"))
            .env("XDG_DATA_DIRS", self.root.join("system"))
            .env("XDG_CURRENT_DESKTOP", "AsaaanTest")
            .env("LANGUAGE", "en")
            .env("LC_ALL", "C.UTF-8")
            .output()
            .unwrap()
    }
    fn names(&self) -> Vec<String> {
        let output = self.run(&[]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let reply: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        reply["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["item"]["title"].as_str().unwrap().to_owned())
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
#[test]
fn xdg_precedence_visibility_and_missing_tryexec() {
    let f = Fixture::new();
    f.entry("system", "override.desktop", "Name=System");
    f.entry("user", "override.desktop", "Name=User");
    f.entry("system", "hidden.desktop", "Name=HiddenSystem");
    f.entry("user", "hidden.desktop", "Hidden=true");
    f.entry(
        "user",
        "nodisplay.desktop",
        "Name=NoDisplay\nNoDisplay=true",
    );
    f.entry(
        "user",
        "only.desktop",
        "Name=OnlyHere\nOnlyShowIn=AsaaanTest;",
    );
    f.entry(
        "user",
        "other.desktop",
        "Name=OtherDesktop\nOnlyShowIn=Other;",
    );
    f.entry("user", "not.desktop", "Name=NotHere\nNotShowIn=AsaaanTest;");
    f.entry(
        "user",
        "try.desktop",
        "Name=MissingBinary\nTryExec=/no/asaaan-binary",
    );
    f.entry("user", "broken.desktop", "Name=Broken\nType=Link");
    assert_eq!(f.names(), vec!["OnlyHere", "User"]);
}
#[test]
fn same_named_apps_keep_distinct_ids_and_nested_desktop_ids() {
    let f = Fixture::new();
    f.entry("user", "one.desktop", "Name=Notes");
    f.entry("user", "browser/two.desktop", "Name=Notes");
    let out = f.run(&[]);
    let data: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(data["total"], 2);
    let ids: Vec<_> = data["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["item"]["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"app:desktop:browser-two.desktop"));
    assert!(
        data["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["origin"].is_string())
    );
}
#[test]
fn rescanning_detects_install_and_remove() {
    let f = Fixture::new();
    assert!(f.names().is_empty());
    f.entry("user", "new.desktop", "Name=New Application");
    assert_eq!(f.names(), vec!["New Application"]);
    std::fs::remove_file(f.root.join("user/applications/new.desktop")).unwrap();
    assert!(f.names().is_empty());
}
#[test]
fn native_exec_expansion_preserves_literal_arguments_and_isolates_stdout() {
    let f = Fixture::new();
    let marker = f.root.join("launch marker.json");
    let exe = env!("CARGO_BIN_EXE_app-catalog-fixture");
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=Probe\nExec=\"{exe}\" record \"{}\" \"literal ; dollar \\\\$value\" %% %c %k %f %U\n",
        marker.display()
    );
    std::fs::write(f.root.join("user/applications/probe.desktop"), entry).unwrap();
    let out = f.run(&["launch", "app:desktop:probe.desktop"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"launched\n");
    let deadline = Instant::now() + Duration::from_secs(5);
    while !marker.exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    let args: Vec<String> = serde_json::from_slice(&std::fs::read(marker).unwrap()).unwrap();
    assert_eq!(args[0], "literal ; dollar $value");
    assert_eq!(args[1], "%");
    assert_eq!(args[2], "Probe");
    assert!(args[3].ends_with("probe.desktop"));
    assert_eq!(args.len(), 4);
}

#[test]
fn gio_observes_install_and_remove_in_one_running_catalog() {
    let f = Fixture::new();
    f.entry("user", "notes.desktop", "Name=Notes");
    let added = f.root.join("user/applications/new.desktop");
    let out = f.run(&["refresh", added.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"refresh passed\n");
}
#[test]
fn removed_desktop_entry_is_rejected_even_in_a_stale_snapshot() {
    let f = Fixture::new();
    f.entry("user", "notes.desktop", "Name=Notes");
    let out = f.run(&["removed", "app:desktop:notes.desktop"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"removed rejected\n");
}
