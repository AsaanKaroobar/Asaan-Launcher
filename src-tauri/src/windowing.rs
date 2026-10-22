//! Wayland compositor integration.
//!
//! Wayland does not let a client place or focus its own toplevels. Hyprland
//! exposes a local IPC socket for actions the user could perform; this module
//! uses it only for this process's own window, identified by PID. Other
//! Wayland compositors keep their normal placement and focus policy.

#[cfg(target_os = "linux")]
use std::time::Duration;
#[cfg(target_os = "linux")]
use tauri::Manager;

pub fn wayland_session() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_SESSION_TYPE").is_ok_and(|value| value == "wayland")
            || std::env::var_os("WAYLAND_DISPLAY").is_some_and(|value| !value.is_empty())
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Ask Hyprland to float, size, center and focus a newly shown window.
///
/// The work happens on a short-lived thread after the GTK map request so the
/// main loop can commit the first Wayland surface. `extent` is the requested
/// logical content size, not the stale outer frame GTK can report pre-map.
pub fn settle_on_show(window: &tauri::WebviewWindow, extent: (u32, u32)) {
    #[cfg(target_os = "linux")]
    {
        if !wayland_session() {
            return;
        }
        let app = window.app_handle().clone();
        let label = window.label().to_owned();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            let visible = || {
                app.get_webview_window(&label)
                    .and_then(|window| window.is_visible().ok())
                    .unwrap_or(false)
            };
            if !visible() {
                return;
            }
            // WebKitGTK can leave a hidden-created surface displaying the frame
            // from before the first application search completed. A one-pixel
            // configure round-trip makes the accelerated compositor commit a
            // fresh buffer on Wayland; the final size is restored immediately.
            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.set_size(tauri::LogicalSize::new(
                    f64::from(extent.0),
                    f64::from(extent.1.saturating_sub(1)),
                ));
                std::thread::sleep(Duration::from_millis(30));
                let _ = window.set_size(tauri::LogicalSize::new(
                    f64::from(extent.0),
                    f64::from(extent.1),
                ));
            }
            if hyprland::available() {
                let _ = hyprland::settle(std::process::id(), extent, &visible);
            }
        });
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (window, extent);
    }
}

#[cfg(target_os = "linux")]
mod hyprland {
    use super::*;
    use std::{
        io::{Read, Write},
        os::unix::net::UnixStream,
        path::PathBuf,
        time::Instant,
    };

    const POLL: Duration = Duration::from_millis(10);
    const TIMEOUT: Duration = Duration::from_millis(750);

    #[derive(Clone, Debug, serde::Deserialize)]
    struct Workspace {
        id: i64,
    }

    #[derive(Clone, Debug, serde::Deserialize)]
    struct Client {
        address: String,
        pid: u32,
        mapped: bool,
        hidden: bool,
        monitor: i64,
        size: [i64; 2],
        workspace: Workspace,
    }

    #[derive(Clone, Debug, serde::Deserialize)]
    struct Monitor {
        id: i64,
        x: i64,
        y: i64,
        width: i64,
        height: i64,
        scale: f64,
        #[serde(default)]
        reserved: Vec<i64>,
    }

    fn socket() -> Option<PathBuf> {
        if !wayland_session() {
            return None;
        }
        let signature = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;
        if signature.is_empty() {
            return None;
        }
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")?;
        Some(
            PathBuf::from(runtime)
                .join("hypr")
                .join(signature)
                .join(".socket.sock"),
        )
    }

    pub(super) fn available() -> bool {
        socket().is_some()
    }

    fn ipc(command: &str) -> Result<String, String> {
        let path = socket().ok_or("Hyprland IPC is unavailable")?;
        let mut stream =
            UnixStream::connect(path).map_err(|error| format!("Hyprland IPC: {error}"))?;
        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_millis(500)))
            .map_err(|error| error.to_string())?;
        stream
            .write_all(command.as_bytes())
            .map_err(|error| error.to_string())?;
        stream
            .shutdown(std::net::Shutdown::Write)
            .map_err(|error| error.to_string())?;
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|error| error.to_string())?;
        Ok(response.trim().to_string())
    }

    fn clients() -> Result<Vec<Client>, String> {
        serde_json::from_str(&ipc("j/clients")?)
            .map_err(|error| format!("Hyprland clients: {error}"))
    }

    fn monitors() -> Result<Vec<Monitor>, String> {
        serde_json::from_str(&ipc("j/monitors")?)
            .map_err(|error| format!("Hyprland monitors: {error}"))
    }

    fn active_workspace() -> Option<i64> {
        let value: serde_json::Value =
            serde_json::from_str(&ipc("j/activeworkspace").ok()?).ok()?;
        value.get("id")?.as_i64()
    }

    fn client_for_pid(clients: &[Client], pid: u32) -> Option<&Client> {
        clients.iter().find(|client| client.pid == pid)
    }

    fn mapped(client: &Client) -> bool {
        client.mapped && !client.hidden
    }

    fn centered(monitor: &Monitor, width: i64, height: i64) -> (i64, i64) {
        let reserved = |index: usize| monitor.reserved.get(index).copied().unwrap_or(0);
        let scale = if monitor.scale > 0.0 {
            monitor.scale
        } else {
            1.0
        };
        let logical_width = monitor.width as f64 / scale;
        let logical_height = monitor.height as f64 / scale;
        let usable_width = logical_width - (reserved(0) + reserved(2)) as f64;
        let usable_height = logical_height - (reserved(1) + reserved(3)) as f64;
        (
            monitor.x + reserved(0) + ((usable_width - width as f64) / 2.0).max(0.0).round() as i64,
            monitor.y
                + reserved(1)
                + ((usable_height - height as f64) / 2.0).max(0.0).round() as i64,
        )
    }

    fn settle_with(pid: u32, extent: (u32, u32), visible: &dyn Fn() -> bool) -> Result<(), String> {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if !visible() {
                return Ok(());
            }
            let listed = clients()?;
            let Some(client) = client_for_pid(&listed, pid).filter(|client| mapped(client)) else {
                if Instant::now() >= deadline {
                    return Ok(());
                }
                std::thread::sleep(POLL);
                continue;
            };
            let address = client.address.clone();
            let workspace = client.workspace.id;
            if !visible() {
                return Ok(());
            }
            if let Some(active) = active_workspace().filter(|active| *active != workspace) {
                let _ = ipc(&format!(
                    "dispatch movetoworkspacesilent {active},address:{address}"
                ));
                std::thread::sleep(POLL);
            }
            let _ = ipc(&format!("dispatch setfloating address:{address}"));
            let _ = ipc(&format!(
                "dispatch resizewindowpixel exact {} {},address:{address}",
                extent.0, extent.1
            ));
            std::thread::sleep(POLL);
            let measured = clients()
                .ok()
                .and_then(|listed| client_for_pid(&listed, pid).cloned())
                .filter(|client| mapped(client) && client.size[0] > 0 && client.size[1] > 0);
            let (width, height) = measured
                .as_ref()
                .map_or((i64::from(extent.0), i64::from(extent.1)), |client| {
                    (client.size[0], client.size[1])
                });
            let monitor = measured.as_ref().and_then(|client| {
                monitors()
                    .ok()?
                    .into_iter()
                    .find(|monitor| monitor.id == client.monitor)
            });
            if let Some(monitor) = monitor {
                let (x, y) = centered(&monitor, width, height);
                if !visible() {
                    return Ok(());
                }
                let _ = ipc(&format!(
                    "dispatch movewindowpixel exact {x} {y},address:{address}"
                ));
            }
            let _ = ipc(&format!("dispatch focuswindow address:{address}"));
            return Ok(());
        }
    }

    pub(super) fn settle(
        pid: u32,
        extent: (u32, u32),
        visible: &dyn Fn() -> bool,
    ) -> Result<(), String> {
        settle_with(pid, extent, visible)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const CLIENTS: &str = r#"[
            {"address":"0xforeign","pid":41,"mapped":true,"hidden":false,"monitor":0,"size":[800,600],"workspace":{"id":2}},
            {"address":"0xlauncher","pid":42,"mapped":true,"hidden":false,"monitor":0,"size":[750,474],"workspace":{"id":2}},
            {"address":"0xunmapped","pid":42,"mapped":false,"hidden":false,"monitor":0,"size":[640,480],"workspace":{"id":2}}
        ]"#;

        #[test]
        fn matches_only_the_requested_process_and_waits_for_a_map() {
            let clients: Vec<Client> = serde_json::from_str(CLIENTS).unwrap();
            assert_eq!(
                client_for_pid(&clients, 42).map(|client| client.address.as_str()),
                Some("0xlauncher")
            );
            assert!(client_for_pid(&clients, 41).is_some_and(mapped));
            assert!(client_for_pid(&clients, 42).is_some_and(mapped));
            assert!(!clients[2].mapped);
            assert!(client_for_pid(&clients, 99).is_none());
        }

        #[test]
        fn ignores_hidden_workspaces() {
            let clients: Vec<Client> = serde_json::from_str(
                r#"[{"address":"0xhidden","pid":7,"mapped":true,"hidden":true,"monitor":0,"size":[750,474],"workspace":{"id":2}}]"#,
            )
            .unwrap();
            assert!(!mapped(&clients[0]));
        }

        #[test]
        fn centers_inside_the_reserved_work_area() {
            let monitor = Monitor {
                id: 0,
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                scale: 1.25,
                reserved: vec![0, 26, 0, 0],
            };
            assert_eq!(centered(&monitor, 750, 474), (393, 208));
        }

        #[test]
        fn centers_negative_origin_and_multi_monitor_boxes() {
            let left = Monitor {
                id: 1,
                x: -1920,
                y: 0,
                width: 1920,
                height: 1080,
                scale: 1.0,
                reserved: vec![0, 0, 0, 0],
            };
            assert_eq!(centered(&left, 750, 474), (-1335, 303));
            let right = Monitor {
                id: 2,
                x: 1920,
                y: 0,
                width: 2560,
                height: 1440,
                scale: 1.0,
                reserved: vec![0, 40, 0, 0],
            };
            assert_eq!(centered(&right, 960, 640), (2720, 420));
        }

        #[test]
        fn oversized_windows_stay_inside_the_work_origin() {
            let monitor = Monitor {
                id: 0,
                x: -800,
                y: 0,
                width: 640,
                height: 480,
                scale: 1.0,
                reserved: vec![0, 0, 0, 0],
            };
            assert_eq!(centered(&monitor, 750, 500), (-800, 0));
        }
    }
}
