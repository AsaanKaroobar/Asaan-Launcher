// One-level bundle scan/duplicate semantics adapted from Floe Catalog.swift.
// Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
use crate::{Application, Target, distinct};
use std::path::{Path, PathBuf};
pub fn roots() -> Vec<PathBuf> {
    let mut roots = vec![
        "/Applications".into(),
        "/System/Applications".into(),
        "/System/Cryptexes/App/System/Applications".into(),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join("Applications"));
    }
    roots
}
pub fn folders(roots: &[PathBuf]) -> Vec<PathBuf> {
    roots
        .iter()
        .flat_map(|root| {
            let mut children: Vec<_> = std::fs::read_dir(root)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.is_dir()
                        && !p
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .starts_with('.')
                        && p.extension().is_none_or(|e| e != "app")
                })
                .collect();
            children.sort();
            let mut all = vec![root.clone()];
            all.extend(children);
            all
        })
        .collect()
}
fn metadata(path: &Path) -> (String, Option<String>) {
    let fallback = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let info = plist::Value::from_file(path.join("Contents/Info.plist")).ok();
    let dict = info.as_ref().and_then(|v| v.as_dictionary());
    let field = |key: &str| {
        dict.and_then(|d| d.get(key))
            .and_then(|v| v.as_string())
            .map(str::to_owned)
    };
    (
        field("CFBundleDisplayName")
            .or_else(|| field("CFBundleName"))
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback),
        field("CFBundleIdentifier"),
    )
}
pub fn scan(roots: &[PathBuf]) -> Vec<Application> {
    let found = folders(roots)
        .iter()
        .flat_map(|root| std::fs::read_dir(root).into_iter().flatten().flatten())
        .map(|e| e.path())
        .filter(|p| {
            p.is_dir()
                && p.extension().is_some_and(|e| e == "app")
                && !p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .starts_with('.')
        })
        .map(|path| {
            let (name, identity) = metadata(&path);
            Application {
                id: format!("app:{}", path.to_string_lossy()),
                name,
                identity,
                origin: None,
                keywords: vec![],
                location: path.clone(),
                target: Target::Bundle(path),
            }
        })
        .collect();
    let mut apps = distinct(found);
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}
