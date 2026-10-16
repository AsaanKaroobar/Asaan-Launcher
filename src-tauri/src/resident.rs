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
    let mut personalization = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())
        .and_then(|dir| asaaan_personalization::Store::open(dir.join("personalization.json")));
    let (control, worker) = process::start_serviced(
        config,
        move |event| {
            if !matches!(
                &event,
                Event::Packet(
                    Packet::PersonalizationRequest { .. } | Packet::PersonalizationReply { .. }
                )
            ) {
                eprintln!("ASAAAN_LIFECYCLE {event:?}");
            }
            let _ = events_tx.send(event);
        },
        move |packet| {
            let Packet::PersonalizationRequest {
                request_id,
                operation,
            } = packet
            else {
                return None;
            };
            let result = match &mut personalization {
                Ok(store) => store.execute(operation, asaaan_personalization::now()),
                Err(error) => Err(error.clone()),
            };
            Some(match result {
                Ok(snapshot) => Packet::PersonalizationReply {
                    request_id: *request_id,
                    snapshot: Some(snapshot),
                    error: None,
                },
                Err(error) => Packet::PersonalizationReply {
                    request_id: *request_id,
                    snapshot: None,
                    error: Some(error),
                },
            })
        },
    );
    app.manage(Resident {
        control: control.clone(),
        worker: Mutex::new(Some(worker)),
    });
    let show = MenuItem::with_id(app, "show", "Show Asaaan Launcher", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide launcher", true, None::<&str>)?;
    let status = MenuItem::with_id(app, "shortcut", &shortcut_status, false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Asaaan Launcher", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &status, &quit])?;
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
                            println!("ASAAAN_RESIDENT_SMOKE_OK CP005 cycles=3 pids={pids:?}");
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
    if smoke || !std::env::args().any(|a| a == "--background" || a == "--hide") {
        control.request(Request::Show);
    }
    Ok(())
}
