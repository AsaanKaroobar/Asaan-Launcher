use asaaan_lifecycle::{Packet, Request, Work, wire};
use std::{
    io::BufReader,
    sync::{Mutex, mpsc},
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};
struct Visibility {
    visible: bool,
    sequence: u64,
    hidden_at: Instant,
    reported: bool,
}
pub struct Panel {
    output: mpsc::Sender<Packet>,
    visibility: Mutex<Visibility>,
    pub work: Work,
    pending: Mutex<
        std::collections::HashMap<
            u64,
            mpsc::Sender<Result<asaaan_personalization::Snapshot, String>>,
        >,
    >,
    request_id: std::sync::atomic::AtomicU64,
}
impl Panel {
    pub fn personalization(
        &self,
        operation: asaaan_personalization::Operation,
    ) -> Result<asaaan_personalization::Snapshot, String> {
        let request_id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let (tx, rx) = mpsc::channel();
        self.pending.lock().unwrap().insert(request_id, tx);
        self.send(Packet::PersonalizationRequest {
            request_id,
            operation,
        });
        let result = rx
            .recv_timeout(Duration::from_secs(10))
            .unwrap_or_else(|_| {
                Err(
                    "Resident personalization request timed out; reopen to check saved state"
                        .into(),
                )
            });
        self.pending.lock().unwrap().remove(&request_id);
        result
    }

    pub fn presented(&self, sequence: u64, focused: bool) -> Result<(), String> {
        let state = self.visibility.lock().unwrap();
        if state.sequence != sequence || !state.visible {
            return Ok(());
        }
        if !focused {
            return Err("The presented search field did not receive focus".into());
        }
        self.send(Packet::Presented {
            sequence,
            visible: true,
        });
        Ok(())
    }
    pub fn focus_gained(&self, app: &tauri::AppHandle) {
        let state = self.visibility.lock().unwrap();
        if state.visible
            && let Some(window) = app.get_webview_window("main")
        {
            let webview: &tauri::Webview = window.as_ref();
            let _ = webview.set_focus();
            let _ = window.emit("panel-shown", state.sequence);
        }
    }
    pub fn focus_lost(&self, app: &tauri::AppHandle) {
        let state = self.visibility.lock().unwrap();
        if !state.visible {
            return;
        }
        let sequence = state.sequence;
        let handle = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            let target = handle.clone();
            let _ = handle.run_on_main_thread(move || {
                let panel = target.state::<Panel>();
                let state = panel.visibility.lock().unwrap();
                let current = state.visible && state.sequence == sequence;
                drop(state);
                // X11 passive shortcut grabs produce transient FocusOut events.
                // A later request supersedes this blur; genuine focus loss still dismisses.
                if current
                    && target
                        .get_webview_window("main")
                        .is_some_and(|w| w.is_focused().is_ok_and(|focused| !focused))
                {
                    let _ = panel.hide(&target);
                }
            });
        });
    }
    pub fn send(&self, packet: Packet) {
        let _ = self.output.send(packet);
    }
    pub fn hide(&self, app: &tauri::AppHandle) -> Result<(), String> {
        if let Some(window) = app.get_webview_window("main") {
            window.hide().map_err(|e| e.to_string())?;
        }
        let mut state = self.visibility.lock().unwrap();
        state.visible = false;
        state.hidden_at = Instant::now();
        state.reported = false;
        self.send(Packet::Presented {
            sequence: state.sequence,
            visible: false,
        });
        Ok(())
    }
    fn receive(
        &self,
        app: &tauri::AppHandle,
        request: Request,
        sequence: u64,
    ) -> Result<(), String> {
        let visible = self.visibility.lock().unwrap().visible;
        {
            let mut state = self.visibility.lock().unwrap();
            state.sequence = sequence;
            state.reported = false;
        }
        if request == Request::Hide || (request == Request::Toggle && visible) {
            return self.hide(app);
        }
        if !matches!(request, Request::Show | Request::Toggle) {
            return Err("Unsupported checkpoint request".into());
        }
        let window = app
            .get_webview_window("main")
            .ok_or("Panel window missing")?;
        window.center().map_err(|e| e.to_string())?;
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        crate::activation::activate(&window).map_err(|e| e.to_string())?;
        let webview: &tauri::Webview = window.as_ref();
        webview.set_focus().map_err(|e| e.to_string())?;
        self.visibility.lock().unwrap().visible = true;
        let _ = window.emit("panel-shown", sequence);
        Ok(())
    }
}
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let (output, packets) = mpsc::channel();
    app.manage(Panel {
        output,
        visibility: Mutex::new(Visibility {
            visible: false,
            sequence: 0,
            hidden_at: Instant::now(),
            reported: false,
        }),
        work: Work::default(),
        pending: Mutex::new(std::collections::HashMap::new()),
        request_id: std::sync::atomic::AtomicU64::new(1),
    });
    std::thread::spawn(move || {
        let mut stdout = std::io::stdout().lock();
        for packet in packets {
            if wire::write(&mut stdout, &packet).is_err() {
                break;
            }
        }
    });
    let handle = app.handle().clone();
    std::thread::spawn(move || {
        let mut input = BufReader::new(std::io::stdin());
        while let Ok(Some(packet)) = wire::read(&mut input) {
            match packet {
                Packet::Request { request, sequence } => {
                    let target = handle.clone();
                    let _ = handle.run_on_main_thread(move || {
                        if let Err(error) =
                            target.state::<Panel>().receive(&target, request, sequence)
                        {
                            eprintln!("Panel request failed: {error}");
                            target.exit(1);
                        }
                    });
                }
                Packet::PersonalizationReply {
                    request_id,
                    snapshot,
                    error,
                } => {
                    if let Some(tx) = handle
                        .state::<Panel>()
                        .pending
                        .lock()
                        .unwrap()
                        .remove(&request_id)
                    {
                        let result = match (snapshot, error) {
                            (Some(state), None) => Ok(state),
                            (_, Some(error)) => Err(error),
                            _ => Err("Invalid resident personalization reply".into()),
                        };
                        let _ = tx.send(result);
                    }
                }
                Packet::Shutdown => {
                    handle.exit(0);
                    return;
                }
                _ => {
                    eprintln!("Invalid resident packet");
                    handle.exit(1);
                    return;
                }
            }
        }
        // An inherited private pipe closing always ends the foreground, even after a parent crash.
        handle.exit(0);
    });
    let handle = app.handle().clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(50));
            let panel = handle.state::<Panel>();
            let mut state = panel.visibility.lock().unwrap();
            if !state.visible
                && state.sequence > 0
                && !state.reported
                && state.hidden_at.elapsed() >= Duration::from_millis(750)
                && !panel.work.is_busy()
            {
                state.reported = true;
                panel.send(Packet::Idle {
                    sequence: state.sequence,
                });
            }
        }
    });
    Ok(())
}
