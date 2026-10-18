mod activation;
mod application_actions;
mod applications;
mod clipboard;
mod data;
mod extension_service;
mod extensions;
mod force_confirmation;
mod panel;
mod picker;
mod process_actions;
#[cfg(any(target_os = "linux", target_os = "windows", test))]
mod process_wait;
#[cfg(target_os = "windows")]
mod process_windows;
#[cfg(target_os = "linux")]
mod process_x11;
mod resident;
mod search_command;
mod settings;
use serde::Serialize;
use tauri::Manager;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    platform: &'static str,
    architecture: &'static str,
    smoke_test: bool,
    shortcut: &'static str,
    application_smoke: bool,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Asaaan Launcher",
        version: env!("CARGO_PKG_VERSION"),
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        smoke_test: smoke_test_requested(),
        application_smoke: application_smoke_requested(),
        shortcut: if std::env::args().any(|a| a == "--shortcut-unavailable") {
            "Shortcut unavailable · use the tray"
        } else {
            "Ctrl+Alt+Space"
        },
    }
}

fn application_smoke_requested() -> bool {
    std::env::args().any(|arg| arg == "--application-smoke-test")
}

fn smoke_test_requested() -> bool {
    std::env::args().any(|arg| arg == "--smoke-test")
}

// CP002 smoke completion waits for the search view, not only the identity reply.
#[tauri::command]
fn frontend_ready(app: tauri::AppHandle) {
    if let Some(panel) = app.try_state::<panel::Panel>() {
        panel.send(asaaan_lifecycle::Packet::Ready);
    }
}
#[tauri::command]
fn panel_presented(app: tauri::AppHandle, sequence: u64, focused: bool) -> Result<(), String> {
    if let Some(panel) = app.try_state::<panel::Panel>() {
        panel.presented(sequence, focused)
    } else {
        Ok(())
    }
}
#[tauri::command]
fn hide_panel(app: tauri::AppHandle, sequence: Option<u64>) -> Result<(), String> {
    if let Some(panel) = app.try_state::<panel::Panel>() {
        match sequence {
            Some(sequence) => panel.hide_requested(&app, sequence),
            None => panel.hide(&app),
        }
    } else {
        app.get_webview_window("main")
            .ok_or("Window unavailable")?
            .hide()
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn search_reference_catalog(app: tauri::AppHandle, query: String) -> Vec<asaaan_search::ResultRow> {
    let panel = app.try_state::<panel::Panel>();
    let _lease = panel.as_ref().map(|p| p.work.begin());
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
        if let Some(panel) = app.try_state::<panel::Panel>() {
            panel.send(asaaan_lifecycle::Packet::SearchVerified);
        } else {
            println!("ASAAAN_NATIVE_SMOKE_OK CP014 query=cal results=Calendar,Calculator");
            app.exit(0);
        }
    }
    Ok(())
}

pub fn run() {
    if let Some(code) = search_command::start() {
        std::process::exit(code);
    }
    if let Some(code) = extensions::start() {
        std::process::exit(code);
    }
    if let Err(error) =
        asaaan_lifecycle::settings::options(&std::env::args().skip(1).collect::<Vec<_>>())
    {
        eprintln!("{error}");
        std::process::exit(64);
    }
    let picker = match picker::start() {
        picker::Start::Normal => None,
        picker::Start::Picker(state) => Some(state),
        picker::Start::Exit(code) => std::process::exit(code),
    };
    let picker_mode = picker.is_some();
    let settings_mode = settings::is_child();
    let managed = std::env::args().any(|a| a == "--panel");
    let resident_smoke = std::env::args().any(|a| a == "--resident-smoke-test");
    let resident_mode =
        !picker_mode && !managed && !smoke_test_requested() && !application_smoke_requested();
    let mut builder = tauri::Builder::default();
    if resident_mode && !resident_smoke {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if args.iter().any(|a| a == "--settings") {
                if let Ok(options) = asaaan_lifecycle::settings::options(&args)
                    && let Some(settings) = app.try_state::<settings::SettingsProcess>()
                {
                    settings.show(options.page);
                }
            } else if args.iter().any(|a| a == "--quit") {
                app.exit(0);
            } else if let Some(resident) = app.try_state::<resident::Resident>() {
                resident
                    .control
                    .request(if args.iter().any(|a| a == "--hide") {
                        asaaan_lifecycle::Request::Hide
                    } else {
                        asaaan_lifecycle::Request::Show
                    });
            }
        }));
    }
    if let Some(picker) = picker {
        builder = builder.manage(picker);
    }
    let application = builder
        .manage(std::sync::Arc::new(asaaan_apps::Store::default()))
        .manage(applications::Launched::default())
        .manage(settings::Closing::default())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            extension_service::extension_exchange,
            settings::open_settings,
            settings::read_settings,
            settings::settings_page_changed,
            settings::close_settings,
            picker::picker_view,
            picker::picker_finish,
            frontend_ready,
            search_reference_catalog,
            search_ready,
            hide_panel,
            panel_presented,
            applications::search_applications,
            applications::launch_application,
            applications::application_icon,
            applications::application_launch_ready,
            application_actions::application_action,
            process_actions::application_process_options,
            process_actions::application_process_action,
            applications::change_personalization
        ])
        .setup(move |app| {
            if resident_mode {
                if std::env::args().any(|a| a == "--quit") {
                    app.handle().exit(0);
                    return Ok(());
                }
                resident::setup(app, resident_smoke)?;
            } else {
                let mut window =
                    tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::default());
                if let Some(state) = app.try_state::<picker::Picker>()
                    && let Some(profile) = state.profile.lock().unwrap().as_ref()
                {
                    window = window.data_directory(profile.path().to_path_buf());
                }
                window
                    .incognito(picker_mode)
                    .initialization_script(if picker_mode {
                        "window.__ASAAAN_PICKER__ = true;"
                    } else if settings_mode {
                        "window.__ASAAAN_SETTINGS__ = true;"
                    } else {
                        ""
                    })
                    .title(if picker_mode {
                        "Asaaan Launcher — Choose"
                    } else if settings_mode {
                        "Asaaan Launcher — Settings"
                    } else {
                        "Asaaan Launcher"
                    })
                    .inner_size(
                        if settings_mode { 960.0 } else { 750.0 },
                        if settings_mode { 640.0 } else { 474.0 },
                    )
                    .min_inner_size(480.0, 420.0)
                    .center()
                    .visible(!managed)
                    .always_on_top(managed && !settings_mode)
                    .build()?;
                if managed {
                    panel::setup(app)?;
                }
                if smoke_test_requested() || application_smoke_requested() {
                    let handle = app.handle().clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(20));
                        eprintln!("Native smoke test timed out");
                        handle.exit(1);
                    });
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let Some(picker) = window.app_handle().try_state::<picker::Picker>() {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        picker.cancel(window.app_handle());
                    }
                    tauri::WindowEvent::Focused(true) => {
                        picker
                            .focused
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    tauri::WindowEvent::Focused(false)
                        if picker.focused.load(std::sync::atomic::Ordering::SeqCst) =>
                    {
                        let app = window.app_handle().clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            if app
                                .get_webview_window("main")
                                .is_some_and(|w| w.is_focused().ok() == Some(false))
                                && let Some(state) = app.try_state::<picker::Picker>()
                            {
                                state.cancel(&app);
                            }
                        });
                    }
                    _ => {}
                }
                return;
            }
            if let Some(panel) = window.app_handle().try_state::<panel::Panel>() {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        if settings::is_child() {
                            let _ = settings::close_settings(window.app_handle().clone());
                        } else {
                            let _ = panel.hide(window.app_handle());
                        }
                    }
                    tauri::WindowEvent::Focused(true) => {
                        panel.focus_gained(window.app_handle());
                    }
                    tauri::WindowEvent::Focused(false) if !settings::is_child() => {
                        panel.focus_lost(window.app_handle());
                    }
                    _ => {}
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("Asaaan Launcher could not start");
    application.run(move |app, event| match event {
        tauri::RunEvent::ExitRequested {
            api, code: None, ..
        } if resident_mode => api.prevent_exit(),
        tauri::RunEvent::Exit => {
            if let Some(state) = app.try_state::<picker::Picker>()
                && let Some(profile) = state.profile.lock().unwrap().take()
                && let Err(e) = profile.close()
            {
                eprintln!("Could not remove temporary picker webview: {e}");
            }

            if let Some(service) = app.try_state::<extension_service::Service>() {
                service.stop();
            }
            if let Some(resident) = app.try_state::<resident::Resident>() {
                resident.stop();
            }
            if let Some(settings) = app.try_state::<settings::SettingsProcess>() {
                settings.stop();
            }
        }
        _ => {}
    });
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
        assert_eq!(value.as_object().unwrap().len(), 7);
    }
}
