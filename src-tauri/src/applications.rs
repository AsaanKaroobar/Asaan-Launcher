use asaaan_apps::Store;
use asaaan_personalization::{Operation, Snapshot};
use std::sync::{Arc, Mutex};
use tauri::Manager;
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationReply {
    #[serde(flatten)]
    catalog: asaaan_apps::SearchReply,
    favorites: Vec<String>,
    queries: Vec<String>,
    remember_searches: bool,
    aliases: std::collections::HashMap<String, String>,
    hidden_results: std::collections::HashMap<String, String>,
    #[serde(rename = "commandTotal")]
    command_total: usize,
}
#[derive(serde::Serialize)]
pub struct LaunchOutcome {
    warning: Option<String>,
}
pub(crate) fn preferences(
    app: &tauri::AppHandle,
    operation: Operation,
) -> Result<Snapshot, String> {
    app.try_state::<crate::panel::Panel>()
        .ok_or("Personalization requires the resident launcher")?
        .personalization(operation)
}
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum PersonalizationChange {
    Favorite { id: String, enabled: bool },
    ClearQueries,
    RememberSearches { enabled: bool },
    Alias { id: String, alias: String },
    Hide { id: String },
    Restore { id: String },
}
impl PersonalizationChange {
    fn resolve(self, store: &Store) -> Result<Operation, String> {
        Ok(match self {
            PersonalizationChange::Favorite { id, enabled } => {
                if id.starts_with("command:") {
                    crate::extension_service::resolve(&id)?;
                } else {
                    store.get(&id)?;
                }
                Operation::Favorite { id, enabled }
            }
            PersonalizationChange::Alias { id, alias } => {
                if !alias.is_empty() {
                    if id.starts_with("command:") {
                        crate::extension_service::resolve(&id)?;
                    } else {
                        store.get(&id)?;
                    }
                }
                Operation::Alias { id, alias }
            }
            PersonalizationChange::Hide { id } => {
                let title = if id.starts_with("command:") {
                    crate::extension_service::resolve(&id)?.title
                } else {
                    store.get(&id)?.name
                };
                Operation::Hide { id, title }
            }
            PersonalizationChange::Restore { id } => Operation::Restore { id },
            PersonalizationChange::ClearQueries => Operation::ClearQueries,
            PersonalizationChange::RememberSearches { enabled } => {
                Operation::RememberSearches { enabled }
            }
        })
    }
}
#[tauri::command]
pub async fn change_personalization(
    app: tauri::AppHandle,
    store: tauri::State<'_, Arc<Store>>,
    operation: PersonalizationChange,
) -> Result<Snapshot, String> {
    let operation = operation.resolve(&store)?;
    let _lease = app
        .try_state::<crate::panel::Panel>()
        .map(|p| p.work.begin());
    tauri::async_runtime::spawn_blocking(move || preferences(&app, operation))
        .await
        .map_err(|e| e.to_string())?
}
#[derive(Default)]
pub struct Launched(pub Mutex<Option<String>>);
#[tauri::command]
pub async fn search_applications(
    app: tauri::AppHandle,
    store: tauri::State<'_, Arc<Store>>,
    query: String,
    refresh: bool,
) -> Result<ApplicationReply, String> {
    let _lease = app
        .try_state::<crate::panel::Panel>()
        .map(|p| p.work.begin());
    let store = Arc::clone(&store);
    tauri::async_runtime::spawn_blocking(move || {
        let (snapshot, warning) = if crate::application_smoke_requested() {
            (Snapshot::default(), None)
        } else {
            match preferences(&app, Operation::Read) {
                Ok(snapshot) => (snapshot, None),
                Err(error) => (
                    Snapshot::default(),
                    Some(format!("Personalization unavailable: {error}")),
                ),
            }
        };
        let commands = if crate::application_smoke_requested() {
            vec![]
        } else {
            crate::extension_service::entries(
                &crate::extension_service::catalog().unwrap_or_default(),
            )
        };
        let extra: Vec<_> = commands
            .iter()
            .map(|e| asaaan_search::Item {
                id: e.id.clone(),
                title: e.title.clone(),
                kind: asaaan_search::ItemKind::Command,
                keywords: e.keywords.clone(),
            })
            .collect();
        let mut catalog = store.search_with_extra(
            &query,
            refresh,
            snapshot.context(&query, asaaan_personalization::now()),
            &extra,
        )?;
        if let Some(warning) = warning {
            catalog.warning = Some(match catalog.warning {
                Some(old) => format!("{old}; {warning}"),
                None => warning,
            });
        }
        Ok(ApplicationReply {
            catalog,
            favorites: snapshot.favorites,
            queries: snapshot.queries,
            remember_searches: snapshot.remember_searches,
            aliases: snapshot.aliases,
            hidden_results: snapshot.hidden_results,
            command_total: commands.len(),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn launch_application(
    app: tauri::AppHandle,
    store: tauri::State<'_, Arc<Store>>,
    id: String,
    query: String,
) -> Result<LaunchOutcome, String> {
    if crate::application_smoke_requested() && id != "app:desktop:asaaan-checkpoint-probe.desktop" {
        return Err("Application smoke only permits its isolated probe".into());
    }
    let _lease = app
        .try_state::<crate::panel::Panel>()
        .map(|p| p.work.begin());
    let store = Arc::clone(&store);
    let target = store.get(&id)?;
    // Floe records root activation before asking the OS to open the application.
    let recorded_id = id.clone();
    let warning = if crate::application_smoke_requested() {
        None
    } else {
        let handle = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            preferences(
                &handle,
                Operation::RecordLaunch {
                    id: recorded_id,
                    query,
                },
            )
        })
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| r)
        .err()
        .map(|error| format!("Application opened, but history could not be saved: {error}"))
    };
    tauri::async_runtime::spawn_blocking(move || asaaan_apps::launch(&target))
        .await
        .map_err(|e| e.to_string())??;
    *app.state::<Launched>().0.lock().unwrap() = Some(id);
    if let Some(warning) = &warning {
        eprintln!("{warning}");
    }
    if warning.is_none()
        && !crate::application_smoke_requested()
        && let Err(error) = crate::hide_panel(app.clone(), None)
    {
        eprintln!("Application launch accepted; panel hide failed: {error}");
    }
    Ok(LaunchOutcome { warning })
}
#[tauri::command]
pub fn application_launch_ready(app: tauri::AppHandle, id: String) -> Result<(), String> {
    if crate::application_smoke_requested() {
        if id != "app:desktop:asaaan-checkpoint-probe.desktop"
            || app.state::<Launched>().0.lock().unwrap().as_ref() != Some(&id)
        {
            return Err("No matching native application launch was accepted".into());
        }
        println!("ASAAAN_APPLICATION_SMOKE_OK CP014 id={id}");
        app.exit(0);
    }
    Ok(())
}
#[tauri::command]
pub fn application_icon(store: tauri::State<'_, Arc<Store>>, id: String) -> Option<String> {
    let app = store.get(&id).ok()?;
    #[cfg(target_os = "linux")]
    {
        use base64::Engine;
        use gio::prelude::*;
        use gtk::prelude::*;
        let icon = asaaan_apps::linux::desktop(&app).ok()?.icon()?;
        let info = gtk::IconTheme::default()?.lookup_by_gicon(
            &icon,
            32,
            gtk::IconLookupFlags::FORCE_SIZE,
        )?;
        let data = info.load_icon().ok()?.save_to_bufferv("png", &[]).ok()?;
        Some(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(data)
        ))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = app;
        None
    }
}

#[cfg(test)]
mod customization_tests {
    use super::*;
    #[test]
    fn public_changes_cannot_forge_titles_or_record_usage() {
        for value in [
            r#"{"kind":"hide","id":"app:a","title":"Forged"}"#,
            r#"{"kind":"recordLaunch","id":"app:a","query":"x"}"#,
            r#"{"kind":"read"}"#,
        ] {
            assert!(serde_json::from_str::<PersonalizationChange>(value).is_err());
        }
    }
    #[test]
    fn mutations_resolve_catalog_titles_and_allow_restoring_missing_apps() {
        let store = Store::new(Arc::new(|| {
            Ok(vec![asaaan_apps::Application {
                id: "app:notes".into(),
                name: "Native Notes".into(),
                location: "/notes".into(),
                identity: None,
                origin: None,
                keywords: vec![],
                target: asaaan_apps::Target::Bundle("/notes".into()),
            }])
        }));
        store.search("", false).unwrap();
        assert_eq!(
            PersonalizationChange::Hide {
                id: "app:notes".into()
            }
            .resolve(&store)
            .unwrap(),
            Operation::Hide {
                id: "app:notes".into(),
                title: "Native Notes".into()
            }
        );
        assert!(
            PersonalizationChange::Alias {
                id: "app:missing".into(),
                alias: "x".into()
            }
            .resolve(&store)
            .is_err()
        );
        assert_eq!(
            PersonalizationChange::Restore {
                id: "app:missing".into()
            }
            .resolve(&store)
            .unwrap(),
            Operation::Restore {
                id: "app:missing".into()
            }
        );
    }
}
