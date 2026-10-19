use asaaan_apps::{
    Application, Target,
    actions::{self, Action, Prepared},
};
fn app(target: Target) -> Application {
    Application {
        id: "app:test".into(),
        name: "A 日本語 ; Notes".into(),
        location: "/forged-unused".into(),
        identity: Some("native.id".into()),
        origin: None,
        keywords: vec![],
        target,
    }
}
#[test]
fn application_menu_offers_metadata_without_copy_file_or_process_actions() {
    let a = app(Target::Bundle(
        std::env::temp_dir().join("System Applications/Missing.app"),
    ));
    let menu = actions::describe(&a);
    assert_eq!(
        menu.iter().map(|a| a.kind).collect::<Vec<_>>(),
        [
            Action::Reveal,
            Action::CopyPath,
            Action::CopyName,
            Action::CopyIdentifier
        ]
    );
    assert!(
        !menu
            .iter()
            .any(|a| ["Copy File", "Quit", "Force Quit…", "Move to Trash…"].contains(&a.label))
    );
}
#[test]
fn literal_metadata_comes_from_catalog_target_and_not_location_or_shell() {
    let a = app(Target::Desktop(
        std::env::temp_dir().join("A 日本語 ; $(false).desktop"),
    ));
    assert_eq!(
        actions::prepare(&a, Action::CopyPath).unwrap(),
        Prepared::Copy {
            text: std::env::temp_dir()
                .join("A 日本語 ; $(false).desktop")
                .to_str()
                .unwrap()
                .to_owned(),
            message: "Copied Path".into()
        }
    );
    assert_eq!(
        actions::prepare(&a, Action::CopyName).unwrap(),
        Prepared::Copy {
            text: a.name.clone(),
            message: format!("Copied {}", a.name)
        }
    );
    assert_eq!(
        actions::prepare(&a, Action::CopyIdentifier).unwrap(),
        Prepared::Copy {
            text: "native.id".into(),
            message: "Copied native.id".into()
        }
    );
}
#[test]
fn windows_id_is_never_exposed_as_a_fake_filesystem_path() {
    let a = app(Target::Windows("Vendor.App!Start".into()));
    assert_eq!(
        actions::describe(&a)
            .iter()
            .map(|a| a.kind)
            .collect::<Vec<_>>(),
        [Action::CopyName, Action::CopyIdentifier]
    );
    assert!(actions::prepare(&a, Action::Reveal).is_err());
    assert!(actions::prepare(&a, Action::CopyPath).is_err());
}
#[test]
fn absent_identifier_and_relative_paths_are_not_offered() {
    let mut a = app(Target::Bundle("relative.app".into()));
    a.identity = None;
    assert_eq!(
        actions::describe(&a)
            .iter()
            .map(|a| a.kind)
            .collect::<Vec<_>>(),
        [Action::CopyName]
    );
    assert!(actions::prepare(&a, Action::CopyIdentifier).is_err());
}
#[test]
fn removed_path_can_be_copied_but_reveal_reports_failure() {
    let a = app(Target::Bundle(
        std::env::temp_dir().join(format!("asaaan-missing-{}.app", std::process::id())),
    ));
    assert!(actions::prepare(&a, Action::CopyPath).is_ok());
    assert!(
        actions::prepare(&a, Action::Reveal)
            .unwrap_err()
            .contains("removed")
    );
}
#[test]
fn reveal_preserves_the_selected_location() {
    let path = std::env::temp_dir().join(format!(
        "asaaan-reveal-{} ; 日本語.desktop",
        std::process::id()
    ));
    std::fs::write(&path, b"fixture").unwrap();
    let a = app(Target::Desktop(path.clone()));
    assert_eq!(
        actions::prepare(&a, Action::Reveal).unwrap(),
        Prepared::Reveal(path.clone())
    );
    std::fs::remove_file(path).unwrap();
}
#[test]
fn unsupported_actions_and_oversize_text_are_rejected() {
    assert!(serde_json::from_str::<Action>("\"trash\"").is_err());
    let mut a = app(Target::Windows("a".into()));
    a.name = "x".repeat(65537);
    assert!(actions::prepare(&a, Action::CopyName).is_err());
    a.name = "x\0y".into();
    assert!(actions::prepare(&a, Action::CopyName).is_err());
}
#[cfg(unix)]
#[test]
fn non_utf8_paths_are_not_copied_lossily() {
    use std::os::unix::ffi::OsStringExt;
    let a = app(Target::Desktop(
        std::ffi::OsString::from_vec(b"/tmp/\xff.desktop".to_vec()).into(),
    ));
    assert!(
        !actions::describe(&a)
            .iter()
            .any(|a| a.kind == Action::CopyPath)
    );
    assert!(actions::prepare(&a, Action::CopyPath).is_err());
}
