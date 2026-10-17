use asaaan_apps::{
    Store,
    actions::{Action, Prepared},
};
use std::sync::Arc;
use tauri::Manager;
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionRequest {
    id: String,
    action: Action,
}
impl ActionRequest {
    fn resolve(self, store: &Store) -> Result<Prepared, String> {
        asaaan_apps::actions::prepare(&store.get(&self.id)?, self.action)
    }
}
#[derive(serde::Serialize)]
pub struct Outcome {
    message: Option<String>,
    dismissed: bool,
}
#[tauri::command]
pub async fn application_action(
    app: tauri::AppHandle,
    store: tauri::State<'_, Arc<Store>>,
    request: ActionRequest,
) -> Result<Outcome, String> {
    let _lease = app
        .try_state::<crate::panel::Panel>()
        .map(|p| p.work.begin());
    let store = Arc::clone(&store);
    let prepared = tauri::async_runtime::spawn_blocking(move || request.resolve(&store))
        .await
        .map_err(|e| e.to_string())??;
    match prepared {
        Prepared::Copy { text, message } => {
            let handle = app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                handle
                    .try_state::<crate::panel::Panel>()
                    .ok_or("Copy requires the resident launcher")?
                    .copy_text(text)
            })
            .await
            .map_err(|e| e.to_string())??;
            Ok(Outcome {
                message: Some(message),
                dismissed: false,
            })
        }
        Prepared::Reveal(path) => {
            tauri::async_runtime::spawn_blocking(move || reveal(&path))
                .await
                .map_err(|e| e.to_string())??;
            crate::hide_panel(app, None)?;
            Ok(Outcome {
                message: None,
                dismissed: true,
            })
        }
    }
}
pub fn reveal(path: &std::path::Path) -> Result<(), String> {
    if !path.is_absolute() || !path.exists() {
        return Err("Application was removed. Refresh the catalog.".into());
    }
    #[cfg(target_os = "linux")]
    {
        use gio::glib::variant::ToVariant;
        use gio::prelude::*;
        let uri = gio::File::for_path(path).uri().to_string();
        let bus = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>)
            .map_err(|_| "File manager session service is unavailable")?;
        bus.call_sync(Some("org.freedesktop.FileManager1"),"/org/freedesktop/FileManager1","org.freedesktop.FileManager1","ShowItems",Some(&(vec![uri],String::new()).to_variant()),None,gio::DBusCallFlags::NONE,5000,None::<&gio::Cancellable>).map_err(|_|"File manager could not reveal the application. Check that a FileManager1 provider is available.".to_string())?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let mut command = std::process::Command::new("/usr/bin/open");
        command.arg("-R").arg(path);
        run_bounded(
            command,
            None,
            std::time::Instant::now() + std::time::Duration::from_secs(5),
        )
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = path;
        Err("Selected-item reveal is unavailable for this application on this platform".into())
    }
}
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn run_bounded(
    mut command: std::process::Command,
    input: Option<String>,
    expires: std::time::Instant,
) -> Result<(), String> {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "The native application action could not start")?;
    let writer = input.map(|text| {
        let mut pipe = child.stdin.take().expect("piped stdin");
        std::thread::spawn(move || pipe.write_all(text.as_bytes()))
    });
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err("The native application action failed".into())
                };
            }
            Ok(None) => {}
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err("Could not read native action status".into());
            }
        }
        if std::time::Instant::now() >= expires {
            let _ = child.kill();
            let _ = child.wait();
            break Err("The native application action timed out".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    if let Some(writer) = writer {
        writer
            .join()
            .map_err(|_| "Native clipboard writer failed")?
            .map_err(|_| "Could not send text to the native clipboard")?;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_actions_accept_only_known_kinds_and_catalog_ids() {
        let store = Store::default();
        for value in [
            r#"{"id":"app:a","action":"copyPath","path":"/etc/passwd"}"#,
            r#"{"id":"app:a","action":"copyName","text":"forged"}"#,
            r#"{"id":"app:a","action":"trash"}"#,
        ] {
            assert!(serde_json::from_str::<ActionRequest>(value).is_err());
        }
        assert!(
            ActionRequest {
                id: "app:missing".into(),
                action: Action::CopyName
            }
            .resolve(&store)
            .is_err()
        );
    }
    #[test]
    fn missing_reveal_is_an_error_without_native_delegation() {
        assert!(
            reveal(&std::env::temp_dir().join("asaaan-no-such-application-cp007.desktop")).is_err()
        );
    }
}
