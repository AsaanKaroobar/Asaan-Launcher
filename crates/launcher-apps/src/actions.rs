//! Scoped application metadata actions, adapted from Floe ItemActions.swift.
//! Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
use crate::{Application, Target};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Reveal,
    CopyPath,
    CopyName,
    CopyIdentifier,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Descriptor {
    pub kind: Action,
    pub label: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prepared {
    Copy { text: String, message: String },
    Reveal(PathBuf),
}
fn path(app: &Application) -> Option<&Path> {
    match &app.target {
        Target::Desktop(path) | Target::Bundle(path) if path.is_absolute() => Some(path),
        _ => None,
    }
}
pub fn describe(app: &Application) -> Vec<Descriptor> {
    let mut actions = vec![];
    if let Some(path) = path(app) {
        actions.push(Descriptor {
            kind: Action::Reveal,
            label: if cfg!(target_os = "macos") {
                "Show in Finder"
            } else {
                "Reveal in File Manager"
            },
        });
        if path.to_str().is_some() {
            actions.push(Descriptor {
                kind: Action::CopyPath,
                label: "Copy Path",
            });
        }
    }
    actions.push(Descriptor {
        kind: Action::CopyName,
        label: "Copy Name",
    });
    if app.identity.as_ref().is_some_and(|s| !s.is_empty()) {
        actions.push(Descriptor {
            kind: Action::CopyIdentifier,
            label: match app.target {
                Target::Desktop(_) => "Copy Desktop Entry ID",
                Target::Bundle(_) => "Copy Bundle Identifier",
                Target::Windows(_) => "Copy Application ID",
            },
        });
    }
    actions
}
pub fn prepare(app: &Application, action: Action) -> Result<Prepared, String> {
    if !describe(app).iter().any(|a| a.kind == action) {
        return Err("This application does not support that action".into());
    }
    let (text, message) = match action {
        Action::Reveal => {
            let path = path(app).ok_or("Application has no filesystem location")?;
            if !path.exists() {
                return Err("Application was removed. Refresh the catalog.".into());
            }
            return Ok(Prepared::Reveal(path.to_path_buf()));
        }
        Action::CopyPath => (
            path(app)
                .and_then(Path::to_str)
                .ok_or("Application path cannot be copied as text")?
                .to_owned(),
            "Copied Path".into(),
        ),
        Action::CopyName => (app.name.clone(), format!("Copied {}", app.name)),
        Action::CopyIdentifier => {
            let id = app
                .identity
                .clone()
                .ok_or("Application has no identifier")?;
            (id.clone(), format!("Copied {id}"))
        }
    };
    if text.is_empty() || text.len() > 64 * 1024 || text.contains('\0') {
        return Err("Application metadata cannot be copied".into());
    }
    Ok(Prepared::Copy { text, message })
}
