use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    platform: &'static str,
    architecture: &'static str,
    smoke_test: bool,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Asaaan Launcher",
        version: env!("CARGO_PKG_VERSION"),
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        smoke_test: smoke_test_requested(),
    }
}

fn smoke_test_requested() -> bool {
    std::env::args().any(|arg| arg == "--smoke-test")
}

// CP002 smoke completion waits for the search view, not only the identity reply.
#[tauri::command]
fn frontend_ready() {}

#[tauri::command]
fn search_reference_catalog(query: String) -> Vec<asaaan_search::ResultRow> {
    asaaan_search::reference_catalog::search_reference(&query)
}

#[tauri::command]
fn search_ready(
    app: tauri::AppHandle,
    query: String,
    result_ids: Vec<String>,
) -> Result<(), String> {
    if smoke_test_requested() {
        let expected: Vec<_> = asaaan_search::reference_catalog::search_reference("cal")
            .into_iter()
            .map(|row| row.item.id)
            .collect();
        if query != "cal" || result_ids != expected || expected.len() != 2 {
            return Err(
                "The installed frontend did not render the expected cal search results".into(),
            );
        }
        println!("ASAAAN_NATIVE_SMOKE_OK CP002 query=cal results=Calendar,Calculator");
        app.exit(0);
    }
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            frontend_ready,
            search_reference_catalog,
            search_ready
        ])
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
        assert_eq!(value.as_object().unwrap().len(), 5);
    }
}
