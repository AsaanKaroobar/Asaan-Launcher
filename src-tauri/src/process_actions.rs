//! Graceful application controls. Catalog identities only; no client-supplied PID or path.
use asaaan_apps::{Application, Store};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Hide,
    Quit,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    id: String,
    action: Action,
    token: String,
}
#[derive(Serialize)]
pub struct Descriptor {
    kind: Action,
    label: &'static str,
    token: String,
}
#[derive(Serialize)]
pub struct Options {
    actions: Vec<Descriptor>,
    notice: Option<String>,
}
#[derive(Serialize)]
pub struct Outcome {
    message: Option<String>,
    dismissed: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Running {
    pub hidden: bool,
    pub can_quit: bool,
    pub token: String,
}
fn describe(running: Option<Running>) -> Vec<Descriptor> {
    let Some(running) = running else {
        return vec![];
    };
    let mut result = vec![];
    if !running.hidden {
        result.push(Descriptor {
            kind: Action::Hide,
            label: "Hide",
            token: running.token.clone(),
        });
    }
    if running.can_quit {
        result.push(Descriptor {
            kind: Action::Quit,
            label: "Quit",
            token: running.token.clone(),
        });
    }
    result
}
#[tauri::command]
pub async fn application_process_options(
    app: tauri::AppHandle,
    store: tauri::State<'_, Arc<Store>>,
    id: String,
) -> Result<Options, String> {
    let _lease = app
        .try_state::<crate::panel::Panel>()
        .map(|p| p.work.begin());
    let target = store.get(&id)?;
    tauri::async_runtime::spawn_blocking(move || match platform(&target, None, None) {
        Ok(running) => Ok(Options {
            actions: describe(running),
            notice: platform_notice(),
        }),
        Err(error) => Ok(Options {
            actions: vec![],
            notice: Some(error),
        }),
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn application_process_action(
    app: tauri::AppHandle,
    store: tauri::State<'_, Arc<Store>>,
    request: Request,
) -> Result<Outcome, String> {
    let _lease = app
        .try_state::<crate::panel::Panel>()
        .map(|p| p.work.begin());
    let target = store.get(&request.id)?;
    let action = request.action;
    let name = target.name.clone();
    tauri::async_runtime::spawn_blocking(move || {
        platform(&target, Some(action), Some(&request.token))
    })
    .await
    .map_err(|e| e.to_string())??;
    if action == Action::Hide {
        crate::hide_panel(app, None)?;
    }
    Ok(Outcome {
        message: if action == Action::Quit {
            Some(format!("Quitting {name}"))
        } else {
            None
        },
        dismissed: action == Action::Hide,
    })
}
fn platform_notice() -> Option<String> {
    if cfg!(any(target_os = "linux", target_os = "windows")) {
        Some("Hide minimizes verified windows. Quit requests that they close; the app can keep running or ask about unsaved work.".into())
    } else {
        None
    }
}
#[cfg(target_os = "linux")]
fn platform(
    app: &Application,
    action: Option<Action>,
    expected: Option<&str>,
) -> Result<Option<Running>, String> {
    super::process_x11::perform(app, action, expected)
}
#[cfg(target_os = "macos")]
fn platform(
    app: &Application,
    action: Option<Action>,
    expected: Option<&str>,
) -> Result<Option<Running>, String> {
    use asaaan_apps::Target;
    use std::{
        io::Read,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    let Target::Bundle(path) = &app.target else {
        return Err("Application has no native bundle identity".into());
    };
    let path =
        std::fs::canonicalize(path).map_err(|_| "Application was removed. Refresh the catalog.")?;
    let mut child = Command::new("/usr/bin/osascript")
        .args(["-l", "JavaScript", "-e", include_str!("process_macos.js")])
        .arg(path)
        .arg(match action {
            None => "inspect",
            Some(Action::Hide) => "hide",
            Some(Action::Quit) => "quit",
        })
        .arg(expected.unwrap_or_default())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "Native application service unavailable")?;
    let expires = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|_| "Could not read native application service")?
        {
            if !status.success() {
                return Err("Application identity changed or the native action was refused".into());
            }
            let mut text = String::new();
            child
                .stdout
                .take()
                .ok_or("Native response unavailable")?
                .take(4096)
                .read_to_string(&mut text)
                .map_err(|_| "Native response unavailable")?;
            if text.trim() == "stopped" {
                return if action.is_none() {
                    Ok(None)
                } else {
                    Err("Application is no longer running".into())
                };
            }
            return serde_json::from_str::<Running>(text.trim())
                .map(Some)
                .map_err(|_| "Native application response invalid".into());
        }
        if Instant::now() >= expires {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "Native application service timed out; check the app before retrying".into(),
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
#[cfg(target_os = "windows")]
fn platform(
    app: &Application,
    action: Option<Action>,
    expected: Option<&str>,
) -> Result<Option<Running>, String> {
    super::process_windows::perform(app, action, expected)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform(
    _app: &Application,
    _action: Option<Action>,
    expected: Option<&str>,
) -> Result<Option<Running>, String> {
    Err("Running-app controls are unavailable until native application identity is supported on this platform".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stopped_apps_never_offer_quit_or_hide() {
        assert!(describe(None).is_empty());
    }
    #[test]
    fn hidden_apps_offer_only_graceful_quit() {
        let d = describe(Some(Running {
            hidden: true,
            can_quit: true,
            token: "fixture".into(),
        }));
        assert_eq!(
            d.iter().map(|d| d.kind).collect::<Vec<_>>(),
            vec![Action::Quit]
        );
    }
    #[test]
    fn visible_apps_offer_hide_then_quit() {
        let d = describe(Some(Running {
            hidden: false,
            can_quit: true,
            token: "fixture".into(),
        }));
        assert_eq!(
            d.iter().map(|d| d.kind).collect::<Vec<_>>(),
            vec![Action::Hide, Action::Quit]
        );
    }
    #[test]
    fn unsupported_close_protocol_never_offers_quit() {
        let d = describe(Some(Running {
            hidden: false,
            can_quit: false,
            token: "fixture".into(),
        }));
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].kind, Action::Hide);
    }
    #[test]
    fn requests_cannot_supply_process_identity_or_force_actions() {
        for text in [
            r#"{"id":"x","action":"quit","pid":123}"#,
            r#"{"id":"x","action":"hide","path":"/app"}"#,
            r#"{"id":"x","action":"forceQuit"}"#,
            r#"{"id":"x","action":"restart"}"#,
        ] {
            assert!(serde_json::from_str::<Request>(text).is_err());
        }
    }
}
