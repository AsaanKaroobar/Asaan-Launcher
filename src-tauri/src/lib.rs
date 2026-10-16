use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    platform: &'static str,
    architecture: &'static str,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Asaaan Launcher",
        version: env!("CARGO_PKG_VERSION"),
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
    }
}

fn smoke_test_requested() -> bool {
    std::env::args().any(|arg| arg == "--smoke-test")
}

// This command has no effect during ordinary use. It confirms that the installed
// frontend mounted and received its native response, not merely that Rust started.
#[tauri::command]
fn frontend_ready(app: tauri::AppHandle) {
    if smoke_test_requested() {
        println!("ASAAAN_NATIVE_SMOKE_OK");
        app.exit(0);
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_app_info, frontend_ready])
        .setup(|app| {
            if smoke_test_requested() {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(20));
                    eprintln!("Native smoke test timed out waiting for the frontend");
                    handle.exit(1);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Asaaan Launcher could not start");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_uses_the_frontend_wire_contract() {
        let value = serde_json::to_value(get_app_info()).unwrap();
        assert_eq!(value["name"], "Asaaan Launcher");
        assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["platform"], std::env::consts::OS);
        assert_eq!(value["architecture"], std::env::consts::ARCH);
        assert_eq!(value.as_object().unwrap().len(), 4);
    }
}
