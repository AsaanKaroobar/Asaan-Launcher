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
}
#[derive(serde::Serialize)]
pub struct LaunchOutcome {
    warning: Option<String>,
}
fn preferences(app: &tauri::AppHandle, operation: Operation) -> Result<Snapshot, String> {
    app.try_state::<crate::panel::Panel>()
        .ok_or("Personalization requires the resident launcher")?
        .personalization(operation)
}
#[tauri::command]
pub async fn change_personalization(
    app: tauri::AppHandle,
    store: tauri::State<'_, Arc<Store>>,
    operation: Operation,
) -> Result<Snapshot, String> {
    match &operation {
        Operation::Favorite { id, .. } => {
            store.get(id)?;
        }
        Operation::ClearQueries | Operation::RememberSearches { .. } => {}
        _ => return Err("Unsupported personalization action".into()),
    }
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
        let mut catalog = store.search_with_context(
            &query,
            refresh,
            snapshot.context(&query, asaaan_personalization::now()),
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
        && let Err(error) = crate::hide_panel(app.clone())
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
        println!("ASAAAN_APPLICATION_SMOKE_OK CP005 id={id}");
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
