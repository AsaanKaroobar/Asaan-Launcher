use asaaan_apps::Store;
use std::sync::{Arc, Mutex};
use tauri::Manager;
#[derive(Default)]
pub struct Launched(pub Mutex<Option<String>>);
#[tauri::command]
pub async fn search_applications(
    app: tauri::AppHandle,
    store: tauri::State<'_, Arc<Store>>,
    query: String,
    refresh: bool,
) -> Result<asaaan_apps::SearchReply, String> {
    let _lease = app
        .try_state::<crate::panel::Panel>()
        .map(|p| p.work.begin());
    let store = Arc::clone(&store);
    tauri::async_runtime::spawn_blocking(move || store.search(&query, refresh))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn launch_application(
    app: tauri::AppHandle,
    store: tauri::State<'_, Arc<Store>>,
    id: String,
) -> Result<(), String> {
    if crate::application_smoke_requested() && id != "app:desktop:asaaan-checkpoint-probe.desktop" {
        return Err("Application smoke only permits its isolated probe".into());
    }
    let _lease = app
        .try_state::<crate::panel::Panel>()
        .map(|p| p.work.begin());
    let store = Arc::clone(&store);
    let selected = id.clone();
    tauri::async_runtime::spawn_blocking(move || asaaan_apps::launch(&store.get(&selected)?))
        .await
        .map_err(|e| e.to_string())??;
    *app.state::<Launched>().0.lock().unwrap() = Some(id);
    if !crate::application_smoke_requested()
        && let Err(error) = crate::hide_panel(app.clone())
    {
        eprintln!("Application launch accepted; panel hide failed: {error}");
    }
    Ok(())
}
#[tauri::command]
pub fn application_launch_ready(app: tauri::AppHandle, id: String) -> Result<(), String> {
    if crate::application_smoke_requested() {
        if id != "app:desktop:asaaan-checkpoint-probe.desktop"
            || app.state::<Launched>().0.lock().unwrap().as_ref() != Some(&id)
        {
            return Err("No matching native application launch was accepted".into());
        }
        println!("ASAAAN_APPLICATION_SMOKE_OK CP004 id={id}");
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
