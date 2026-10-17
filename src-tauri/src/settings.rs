use asaaan_lifecycle::{
    Packet, Request,
    settings::{Page, Step},
};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use tauri::Manager;
pub fn is_child() -> bool {
    std::env::args().any(|a| a == "--settings") && std::env::args().any(|a| a == "--panel")
}
pub struct SettingsProcess {
    pub control: asaaan_lifecycle::process::Controller,
    pub worker: Mutex<Option<std::thread::JoinHandle<()>>>,
    pub pid: std::sync::Arc<AtomicU32>,
    pub last_page: Mutex<Page>,
}
impl SettingsProcess {
    pub fn show(&self, page: Option<Page>) {
        let last = *self.last_page.lock().unwrap();
        match asaaan_lifecycle::settings::step(
            self.pid.load(Ordering::SeqCst) != 0,
            page,
            Some(last),
        ) {
            Step::Start(page) | Step::Forward(Some(page)) => {
                self.control.request(Request::BuiltIn(page.id().into()))
            }
            Step::Forward(None) => self.control.request(Request::Show),
        }
    }
    pub fn stop(&self) {
        self.control.stop();
        if let Some(worker) = self.worker.lock().unwrap().take() {
            let _ = worker.join();
        }
    }
}
#[derive(Default)]
pub struct Closing(AtomicBool);
#[tauri::command]
pub fn open_settings(app: tauri::AppHandle, page: Option<Page>) -> Result<(), String> {
    if let Some(resident) = app.try_state::<SettingsProcess>() {
        resident.show(page);
        return Ok(());
    }
    let panel = app
        .try_state::<crate::panel::Panel>()
        .ok_or("Settings require the resident launcher")?;
    panel.send(Packet::OpenSettings { page });
    Ok(())
}
#[tauri::command]
pub fn settings_page_changed(app: tauri::AppHandle, page: Page) -> Result<(), String> {
    if !is_child() {
        return Err("Not a settings process".into());
    }
    app.state::<crate::panel::Panel>()
        .send(Packet::SettingsPage { page });
    Ok(())
}
#[tauri::command]
pub fn close_settings(app: tauri::AppHandle) -> Result<(), String> {
    if !is_child() {
        return Err("Not a settings process".into());
    }
    if app.state::<Closing>().0.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    app.state::<crate::panel::Panel>().hide(&app)?;
    std::thread::spawn(move || {
        loop {
            let busy = app.state::<crate::panel::Panel>().work.is_busy();
            if asaaan_lifecycle::settings::exit_due(false, busy) {
                app.exit(0);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    });
    Ok(())
}
#[derive(serde::Serialize)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub installed: bool,
}
#[derive(serde::Serialize)]
pub struct View {
    pub snapshot: asaaan_personalization::Snapshot,
    pub applications: Vec<Application>,
    pub warning: Option<String>,
}
#[tauri::command]
pub async fn read_settings(
    app: tauri::AppHandle,
    store: tauri::State<'_, std::sync::Arc<asaaan_apps::Store>>,
) -> Result<View, String> {
    if !is_child() {
        return Err("Not a settings process".into());
    }
    let _lease = app.state::<crate::panel::Panel>().work.begin();
    let store = std::sync::Arc::clone(&store);
    tauri::async_runtime::spawn_blocking(move || {
        let snapshot =
            crate::applications::preferences(&app, asaaan_personalization::Operation::Read)?;
        let catalog = store.search("", true)?;
        let mut applications: Vec<_> = catalog
            .rows
            .into_iter()
            .map(|row| Application {
                id: row.item.id,
                name: row.item.title,
                installed: true,
            })
            .collect();
        for id in snapshot
            .aliases
            .keys()
            .chain(snapshot.hidden_results.keys())
        {
            if !applications.iter().any(|a| &a.id == id) {
                applications.push(Application {
                    id: id.clone(),
                    name: snapshot
                        .hidden_results
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| id.clone()),
                    installed: false,
                });
            }
        }
        applications.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then(a.id.cmp(&b.id))
        });
        Ok(View {
            snapshot,
            applications,
            warning: catalog.warning,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
