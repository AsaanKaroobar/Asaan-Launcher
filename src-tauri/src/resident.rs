use asaaan_lifecycle::{
    Packet, Request,
    process::{self, Config, Controller, Event},
};
use std::sync::{Mutex, mpsc};
use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
pub struct Resident {
    pub control: Controller,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
}
impl Resident {
    pub fn stop(&self) {
        self.control.stop();
        if let Some(worker) = self.worker.lock().unwrap().take() {
            let _ = worker.join();
        }
    }
}
pub fn setup(app: &mut tauri::App, smoke: bool) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
    let wayland = std::env::var("XDG_SESSION_TYPE").is_ok_and(|s| s == "wayland");
    let shortcut_status = if wayland {
        "Global shortcut unavailable on Wayland; use the tray".to_string()
    } else {
        match app.handle().plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, key, event| {
                    if key == &shortcut
                        && event.state() == ShortcutState::Released
                        && let Some(resident) = app.try_state::<Resident>()
                    {
                        resident.control.request(Request::Toggle);
                    }
                })
                .build(),
        ) {
            Ok(()) => match app.global_shortcut().register(shortcut) {
                Ok(()) => "Ctrl+Alt+Space".into(),
                Err(error) => format!("Shortcut unavailable: {error}"),
            },
            Err(error) => format!("Shortcut unavailable: {error}"),
        }
    };
    eprintln!("ASAAAN_SHORTCUT {shortcut_status}");
    let mut config = Config::panel(std::env::current_exe()?);
    if smoke {
        config.arguments.push("--smoke-test".into());
    }
    if shortcut_status != "Ctrl+Alt+Space" {
        config.arguments.push("--shortcut-unavailable".into());
    }
    let (events_tx, events) = mpsc::channel();
    let personalization = std::sync::Arc::new(Mutex::new(
        crate::data::personalization_path().and_then(asaaan_personalization::Store::open),
    ));
    app.manage(crate::extension_service::Service::start(
        app.handle().clone(),
        personalization.clone(),
    ));
    let panel_preferences = personalization.clone();
    let clipboard_app = app.handle().clone();
    let (control, worker) = process::start_serviced(
        config,
        move |event| {
            if !matches!(
                &event,
                Event::Packet(
                    Packet::ExtensionRequest { .. }
                        | Packet::ExtensionReply { .. }
                        | Packet::PersonalizationRequest { .. }
                        | Packet::PersonalizationReply { .. }
                        | Packet::ClipboardWrite { .. }
                        | Packet::ClipboardWritten { .. }
                )
            ) {
                eprintln!("ASAAAN_LIFECYCLE {event:?}");
            }
            let _ = events_tx.send(event);
        },
        move |packet| serve(&clipboard_app, &panel_preferences, packet),
    );
    app.manage(Resident {
        control: control.clone(),
        worker: Mutex::new(Some(worker)),
    });
    let mut settings_config = Config::panel(std::env::current_exe()?);
    settings_config.arguments.push("--settings".into());
    let settings_pid = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let observed_pid = settings_pid.clone();
    let settings_app = app.handle().clone();
    let (settings_control, settings_worker) = process::start_serviced(
        settings_config,
        move |event| {
            match &event {
                Event::Started(pid) => {
                    observed_pid.store(*pid, std::sync::atomic::Ordering::SeqCst);
                }
                Event::Exited(_) => {
                    observed_pid.store(0, std::sync::atomic::Ordering::SeqCst);
                }
                _ => {}
            }
            if !private_packet(&event) {
                eprintln!("ASAAAN_SETTINGS {event:?}");
            }
        },
        move |packet| serve(&settings_app, &personalization, packet),
    );
    app.manage(crate::settings::SettingsProcess {
        control: settings_control,
        worker: Mutex::new(Some(settings_worker)),
        pid: settings_pid,
        last_page: Mutex::new(Default::default()),
    });
    let show = MenuItem::with_id(app, "show", "Show Asaaan Launcher", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide launcher", true, None::<&str>)?;
    let status = MenuItem::with_id(app, "shortcut", &shortcut_status, false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Asaaan Launcher", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &settings, &status, &quit])?;
    TrayIconBuilder::new()
        .icon(
            app.default_window_icon()
                .ok_or("Application icon missing")?
                .clone(),
        )
        .tooltip("Asaaan Launcher")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => app.state::<Resident>().control.request(Request::Show),
            "hide" => app.state::<Resident>().control.request(Request::Hide),
            "settings" => app.state::<crate::settings::SettingsProcess>().show(None),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    if smoke {
        let handle = app.handle().clone();
        let smoke_control = control.clone();
        std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
            let mut cycles = 0;
            let mut verified = false;
            let mut shown = false;
            let mut hiding = false;
            let mut pids = Vec::new();
            loop {
                let event = events.recv_timeout(std::time::Duration::from_millis(100));
                match event {
                    Ok(Event::Started(pid)) => {
                        pids.push(pid);
                        verified = false;
                        shown = false;
                        hiding = false;
                    }
                    Ok(Event::Packet(Packet::SearchVerified)) => verified = true,
                    Ok(Event::Packet(Packet::Presented { visible: true, .. })) => shown = true,
                    Ok(Event::Exited(_)) => {
                        if !verified || !shown || !hiding {
                            eprintln!("Panel exited before smoke verification");
                            handle.exit(1);
                            return;
                        }
                        cycles += 1;
                        if cycles == 3 {
                            println!("ASAAAN_RESIDENT_SMOKE_OK CP014 cycles=3 pids={pids:?}");
                            handle.exit(0);
                            return;
                        }
                        smoke_control.request(Request::Show);
                    }
                    Ok(Event::Failed(error)) => {
                        eprintln!("Resident smoke failed: {error}");
                        handle.exit(1);
                        return;
                    }
                    _ => {}
                }
                if verified && shown && !hiding {
                    hiding = true;
                    smoke_control.request(Request::Hide);
                }
                if std::time::Instant::now() > deadline {
                    eprintln!("Resident smoke timed out");
                    handle.exit(1);
                    return;
                }
            }
        });
    }
    let options =
        asaaan_lifecycle::settings::options(&std::env::args().skip(1).collect::<Vec<_>>())
            .map_err(std::io::Error::other)?;
    if options.requested {
        app.state::<crate::settings::SettingsProcess>()
            .show(options.page);
    }
    if smoke
        || (!options.requested && !std::env::args().any(|a| a == "--background" || a == "--hide"))
    {
        control.request(Request::Show);
    }
    Ok(())
}

fn private_packet(event: &Event) -> bool {
    matches!(
        event,
        Event::Packet(
            Packet::ExtensionRequest { .. }
                | Packet::ExtensionReply { .. }
                | Packet::PersonalizationRequest { .. }
                | Packet::PersonalizationReply { .. }
                | Packet::ClipboardWrite { .. }
                | Packet::ClipboardWritten { .. }
        )
    )
}
fn serve(
    app: &tauri::AppHandle,
    store: &Mutex<Result<asaaan_personalization::Store, String>>,
    packet: &Packet,
) -> Option<Packet> {
    match packet {
        Packet::Presented { visible, .. } => {
            if let Some(service) = app.try_state::<crate::extension_service::Service>() {
                service.control.visible(*visible);
            }
            None
        }
        Packet::ExtensionRequest {
            request_id,
            operation,
        } => {
            let result = app
                .try_state::<crate::extension_service::Service>()
                .ok_or("Extension service unavailable".to_string())
                .and_then(|s| s.control.exchange(operation.clone()));
            Some(match result {
                Ok(snapshot) => Packet::ExtensionReply {
                    request_id: *request_id,
                    snapshot: Some(Box::new(snapshot)),
                    error: None,
                },
                Err(error) => Packet::ExtensionReply {
                    request_id: *request_id,
                    snapshot: None,
                    error: Some(error),
                },
            })
        }
        Packet::OpenSettings { page } => {
            if let Some(settings) = app.try_state::<crate::settings::SettingsProcess>() {
                settings.show(*page);
            }
            None
        }
        Packet::SettingsPage { page } => {
            if let Some(settings) = app.try_state::<crate::settings::SettingsProcess>() {
                *settings.last_page.lock().unwrap() = *page;
            }
            None
        }
        Packet::ClipboardWrite {
            request_id,
            text,
            deadline,
        } => Some(Packet::ClipboardWritten {
            request_id: *request_id,
            error: crate::clipboard::write(app, text.clone(), *deadline).err(),
        }),
        Packet::PersonalizationRequest {
            request_id,
            operation,
        } => {
            let result = match &mut *store.lock().unwrap() {
                Ok(store) => store.execute(operation, asaaan_personalization::now()),
                Err(error) => Err(error.clone()),
            };
            Some(match result {
                Ok(snapshot) => Packet::PersonalizationReply {
                    request_id: *request_id,
                    snapshot: Some(Box::new(snapshot)),
                    error: None,
                },
                Err(error) => Packet::PersonalizationReply {
                    request_id: *request_id,
                    snapshot: None,
                    error: Some(error),
                },
            })
        }
        _ => None,
    }
}
