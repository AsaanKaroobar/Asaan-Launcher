//! The resident owns one interactive host and its resumable List state; the panel only uses typed RPC.
use asaaan_extensions::{
    manifest::{self, Command as ExtensionCommand, Platform},
    protocol::{Decoder, Message},
    view::{Entry, Model, Operation, Snapshot, Toast},
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use tauri::Manager;
pub fn catalog() -> Result<Vec<ExtensionCommand>, String> {
    let root = crate::extensions::resources()?;
    Ok(
        manifest::scan(&root.join("extensions"), Platform::current())
            .into_iter()
            .filter(|c| {
                c.mode == "view"
                    && (c.extension_name == "list-lab"
                        || (c.extension_name == "diagnostics"
                            && ["throws", "rejects", "exits", "hangs"].contains(&c.name.as_str())))
                    && c.arguments.is_empty()
                    && !c
                        .extension_preferences
                        .iter()
                        .chain(&c.command_preferences)
                        .any(|p| p.required)
            })
            .collect(),
    )
}
pub fn entries(commands: &[ExtensionCommand]) -> Vec<Entry> {
    commands
        .iter()
        .map(|c| Entry {
            id: format!("command:{}", c.id),
            title: c.title.clone(),
            extension: c.extension_title.clone(),
            keywords: c.keywords.clone(),
        })
        .collect()
}
pub fn resolve(id: &str) -> Result<ExtensionCommand, String> {
    catalog()?
        .into_iter()
        .find(|c| format!("command:{}", c.id) == id)
        .ok_or("Extension command is no longer available.".into())
}
enum Input {
    Exchange(Operation, mpsc::SyncSender<Result<Snapshot, String>>),
}
#[derive(Clone)]
pub struct Controller {
    input: mpsc::SyncSender<Input>,
    stopped: Arc<AtomicBool>,
    visible: Arc<AtomicBool>,
}
impl Controller {
    pub fn exchange(&self, op: Operation) -> Result<Snapshot, String> {
        if self.stopped.load(Ordering::Relaxed) {
            return Err("The extension service is stopping.".into());
        }
        let (tx, rx) = mpsc::sync_channel(1);
        self.input
            .try_send(Input::Exchange(op, tx))
            .map_err(|_| "The extension service is busy.".to_string())?;
        rx.recv_timeout(Duration::from_secs(8))
            .map_err(|_| "The extension service did not answer. Reopen the launcher.".to_string())?
    }
    pub fn visible(&self, visible: bool) {
        self.visible.store(visible, Ordering::Relaxed);
    }
}
pub struct Service {
    pub control: Controller,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
}
impl Service {
    pub fn start(
        app: tauri::AppHandle,
        preferences: Arc<Mutex<Result<asaaan_personalization::Store, String>>>,
    ) -> Self {
        let (input, rx) = mpsc::sync_channel(32);
        let stopped = Arc::new(AtomicBool::new(false));
        let visibility = Arc::new(AtomicBool::new(false));
        let control = Controller {
            input,
            stopped: stopped.clone(),
            visible: visibility.clone(),
        };
        let worker = std::thread::spawn(move || {
            let commands = catalog();
            let entries = commands.as_ref().map(|c| entries(c)).unwrap_or_default();
            let mut active: Option<Active> = None;
            let mut next_token = 1;
            let clock = Instant::now();
            let mut resume = asaaan_extensions::resume::Resume::default();
            let mut visible = false;
            while !stopped.load(Ordering::Relaxed) {
                let next_visible = visibility.load(Ordering::Relaxed);
                if next_visible != visible {
                    visible = next_visible;
                    resume.visible(visible, active.is_some(), clock.elapsed());
                }
                match rx.recv_timeout(Duration::from_millis(10)) {
                    Ok(Input::Exchange(op, reply)) => {
                        let result = (|| -> Result<Snapshot, String> {
                            match op {
                                Operation::Read => {}
                                Operation::Retry { token } => {
                                    let entry = target(&mut active, token)?.entry.clone();
                                    let command = commands
                                        .as_ref()
                                        .map_err(Clone::clone)?
                                        .iter()
                                        .find(|c| format!("command:{}", c.id) == entry.id)
                                        .ok_or("Extension command unavailable.")?;
                                    active = None;
                                    let child = Worker::start(command)?;
                                    active = Some(Active {
                                        token: next_token,
                                        entry,
                                        model: Model::default(),
                                        worker: child,
                                        toast_expires: None,
                                        suppress_hud: false,
                                    });
                                    next_token += 1;
                                    resume.started(clock.elapsed());
                                }
                                Operation::OpenMenu {
                                    token,
                                    id,
                                    revision,
                                    query,
                                } => {
                                    let a = target(&mut active, token)?;
                                    if let Some(message) =
                                        a.model.open_menu(id, revision, &query)?
                                    {
                                        a.worker.send(message)?;
                                    }
                                }
                                Operation::Start { id, query } => {
                                    if query.len() > 4096 {
                                        return Err("Root search is too long.".into());
                                    }
                                    let command = commands
                                        .as_ref()
                                        .map_err(Clone::clone)?
                                        .iter()
                                        .find(|c| format!("command:{}", c.id) == id)
                                        .ok_or("Extension command is unavailable.")?;
                                    let entry = entries
                                        .iter()
                                        .find(|e| e.id == id)
                                        .ok_or("Extension command is unavailable.")?
                                        .clone();
                                    // A duplicate activation resumes this command instead of forking a second host.
                                    if active.as_ref().is_none_or(|a| a.entry.id != id) {
                                        match &mut *preferences.lock().unwrap() {
                                            Ok(store) => {
                                                store.execute(&asaaan_personalization::Operation::RecordLaunch{id,query},asaaan_personalization::now())?;
                                            }
                                            Err(error) => return Err(error.clone()),
                                        };
                                        let child = Worker::start(command)?;
                                        active = Some(Active {
                                            token: next_token,
                                            entry,
                                            model: Model::default(),
                                            worker: child,
                                            toast_expires: None,
                                            suppress_hud: false,
                                        });
                                        next_token = next_token
                                            .checked_add(1)
                                            .ok_or("Extension session counter exhausted.")?;
                                    }
                                    resume.started(clock.elapsed());
                                }
                                Operation::Search { token, text } => {
                                    let a = target(&mut active, token)?;
                                    if let Some(message) = a.model.search(text)? {
                                        a.worker.send(message)?;
                                    }
                                }
                                Operation::Select { token, id, query } => {
                                    target(&mut active, token)?.model.select(id, &query)?;
                                }
                                Operation::Run {
                                    token,
                                    id,
                                    revision,
                                    query,
                                } => {
                                    let a = target(&mut active, token)?;
                                    let message = a.model.run(id, revision, &query)?;
                                    a.suppress_hud = false;
                                    a.worker.send(message)?;
                                }
                                Operation::Back { token } => {
                                    let a = target(&mut active, token)?;
                                    if a.model.failure.is_some() {
                                        active = None;
                                    } else {
                                        a.worker.send(json!({"type":"pop"}))?;
                                    }
                                }
                                Operation::Stop { token } => {
                                    target(&mut active, token)?;
                                    active = None;
                                }
                            }
                            snapshot(&active, &entries)
                        })();
                        let _ = reply.send(result);
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    _ => {}
                }
                if resume.expired(clock.elapsed()) {
                    active = None;
                    resume.ended();
                }
                let mut end = false;
                if let Some(a) = &mut active {
                    for _ in 0..64 {
                        match a.worker.receive() {
                            Ok(Some(Message::Render(tree))) => a.model.render(tree),
                            Ok(Some(Message::UnresolvedRender)) => {
                                if let Err(e) = a.worker.send(json!({"type":"fullRender"})) {
                                    a.model.fail("error", e);
                                }
                            }
                            Ok(Some(Message::Fields(fields))) => {
                                let kind = fields.get("type").and_then(Value::as_str).unwrap_or("");
                                match kind {
                                    "started" => {
                                        eprintln!(
                                            "ASAAAN_EXTENSION_HOST token={} supervisor={} pid={}",
                                            a.token,
                                            a.worker.child.id(),
                                            fields.get("pid").and_then(Value::as_u64).unwrap_or(0)
                                        );
                                    }
                                    "exit" | "popToRoot" => {
                                        end = true;
                                        break;
                                    }
                                    "close" => hide(&app),
                                    "hud" => {
                                        if !a.suppress_hud {
                                            a.model.toast = Some(Toast {
                                                id: -2,
                                                style: "success".into(),
                                                title: fields
                                                    .get("title")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or("")
                                                    .into(),
                                                message: None,
                                            });
                                            a.model.revision += 1;
                                            a.toast_expires =
                                                Some(Instant::now() + Duration::from_secs(3));
                                            hide(&app);
                                        }
                                    }
                                    "copy" => {
                                        let result = if fields
                                            .get("html")
                                            .is_some_and(|v| !v.is_null())
                                            || fields.get("file").is_some_and(|v| !v.is_null())
                                            || fields.get("concealed").and_then(Value::as_bool)
                                                == Some(true)
                                        {
                                            Err("Rich or concealed extension copies are unavailable in this checkpoint.".into())
                                        } else {
                                            crate::clipboard::write(
                                                &app,
                                                fields
                                                    .get("text")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or("")
                                                    .into(),
                                                asaaan_lifecycle::clipboard::now()
                                                    + asaaan_lifecycle::clipboard::TIMEOUT_MS,
                                            )
                                        };
                                        if let Err(e) = result {
                                            a.suppress_hud = true;
                                            a.error_toast(e);
                                        }
                                    }
                                    "request" => {
                                        if let Some(id) = fields.get("id").and_then(Value::as_i64)
                                            && let Err(e)=a.worker.send(json!({"type":"reply","id":id,"error":"This extension API is unavailable in checkpoint 14."})){a.model.fail("error",e);}
                                    }
                                    "unresponsive" => {
                                        if a.model.failure.is_none() {
                                            a.model.fail(
                                                "unresponsive",
                                                "The extension isn't responding.",
                                            );
                                        }
                                    }
                                    "pong" => a.model.pong(),
                                    "ended" => {
                                        if fields.get("success").and_then(Value::as_bool)
                                            == Some(true)
                                        {
                                            end = true;
                                        } else if a.model.failure.is_none() {
                                            a.model.fail(
                                                "crashed",
                                                format!(
                                                    "The extension stopped unexpectedly (exit {}).",
                                                    fields
                                                        .get("code")
                                                        .map(Value::to_string)
                                                        .unwrap_or("unknown".into())
                                                ),
                                            );
                                        }
                                        break;
                                    }
                                    "paste" | "open" | "openPreferences" | "signIn" => a
                                        .error_toast(
                                        "This extension action is unavailable in checkpoint 14."
                                            .into(),
                                    ),
                                    _ => {
                                        if a.suppress_hud
                                            && kind == "toast"
                                            && fields.get("style").and_then(Value::as_str)
                                                != Some("failure")
                                        {
                                            continue;
                                        }
                                        a.model.fields(&fields);
                                        if kind == "toast" && fields.get("hidden").and_then(Value::as_bool)!=Some(true) {
                                            a.toast_expires = if a
                                                .model
                                                .toast
                                                .as_ref()
                                                .is_some_and(|t| t.style != "animated")
                                                && !fields.contains_key("primaryTitle")
                                                && !fields.contains_key("secondaryTitle")
                                            {
                                                Some(Instant::now() + Duration::from_secs(3))
                                            } else {
                                                None
                                            };
                                        }
                                    }
                                }
                            }
                            Ok(None) => break,
                            Err(error) => {
                                if a.model.failure.is_none() {
                                    a.model.fail("crashed", error);
                                }
                                break;
                            }
                        }
                    }
                    if a.toast_expires.is_some_and(|at| Instant::now() >= at) {
                        a.model.toast = None;
                        a.model.revision += 1;
                        a.toast_expires = None;
                    }
                }
                if end {
                    active = None;
                    resume.ended();
                }
            }
            drop(active); // closing the supervisor's input also cleans a nonresponsive Bun process.
        });
        Self {
            control,
            worker: Mutex::new(Some(worker)),
        }
    }
    pub fn stop(&self) {
        self.control.stopped.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.lock().unwrap().take() {
            let _ = worker.join();
        }
    }
}
fn hide(app: &tauri::AppHandle) {
    if let Some(resident) = app.try_state::<crate::resident::Resident>() {
        resident.control.request(asaaan_lifecycle::Request::Hide);
    }
}
struct Active {
    token: u64,
    entry: Entry,
    model: Model,
    worker: Worker,
    toast_expires: Option<Instant>,
    suppress_hud: bool,
}
impl Active {
    fn error_toast(&mut self, message: String) {
        self.model.toast = Some(Toast {
            id: -1,
            style: "failure".into(),
            title: "Extension action failed".into(),
            message: Some(message),
        });
        self.model.revision += 1;
        self.toast_expires = Some(Instant::now() + Duration::from_secs(3));
    }
}
fn target(active: &mut Option<Active>, token: u64) -> Result<&mut Active, String> {
    active
        .as_mut()
        .filter(|a| a.token == token)
        .ok_or("This extension session has ended or was replaced.".into())
}
fn snapshot(active: &Option<Active>, commands: &[Entry]) -> Result<Snapshot, String> {
    Ok(Snapshot {
        commands: commands.to_vec(),
        active: active
            .as_ref()
            .map(|a| a.model.snapshot(a.token, &a.entry))
            .transpose()?,
    })
}
enum Chunk {
    Bytes(Vec<u8>),
    Closed,
    Error,
}
struct Worker {
    child: Child,
    input: Option<mpsc::SyncSender<Value>>,
    output: mpsc::Receiver<Chunk>,
    decoder: Decoder,
    pending: VecDeque<Message>,
    closed: bool,
}
impl Worker {
    fn start(command: &ExtensionCommand) -> Result<Self, String> {
        let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .args(["--extension-worker", &command.extension_name, &command.name])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        let mut stdin = child.stdin.take().ok_or("Supervisor input unavailable")?;
        let mut stdout = child.stdout.take().ok_or("Supervisor output unavailable")?;
        let (input, writes) = mpsc::sync_channel::<Value>(16);
        std::thread::spawn(move || {
            while let Ok(value) = writes.recv() {
                let Ok(mut bytes) = serde_json::to_vec(&value) else {
                    break;
                };
                bytes.push(b'\n');
                if stdin.write_all(&bytes).is_err() {
                    break;
                }
            }
        });
        let (tx, output) = mpsc::sync_channel(16);
        std::thread::spawn(move || {
            let mut bytes = [0; 8192];
            loop {
                match stdout.read(&mut bytes) {
                    Ok(0) => {
                        let _ = tx.send(Chunk::Closed);
                        break;
                    }
                    Ok(n) => {
                        if tx.send(Chunk::Bytes(bytes[..n].to_vec())).is_err() {
                            break;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        let _ = tx.send(Chunk::Error);
                        break;
                    }
                }
            }
        });
        Ok(Self {
            child,
            input: Some(input),
            output,
            decoder: Decoder::default(),
            pending: VecDeque::new(),
            closed: false,
        })
    }
    fn send(&self, value: Value) -> Result<(), String> {
        self.input
            .as_ref()
            .ok_or("Extension input is closed")?
            .try_send(value)
            .map_err(|_| "Extension input is closed or busy.".into())
    }
    fn receive(&mut self) -> Result<Option<Message>, String> {
        if let Some(message) = self.pending.pop_front() {
            return Ok(Some(message));
        }
        match self.output.try_recv() {
            Ok(Chunk::Bytes(bytes)) => {
                self.pending.extend(
                    self.decoder
                        .append(&bytes)
                        .map_err(|_| "The extension sent an invalid or oversized view.")?,
                );
                Ok(self.pending.pop_front())
            }
            Ok(Chunk::Closed) => {
                self.closed = true;
                Err("The extension supervisor stopped unexpectedly.".into())
            }
            Ok(Chunk::Error) => {
                self.closed = true;
                Err("The extension supervisor pipe failed.".into())
            }
            Err(mpsc::TryRecvError::Disconnected) if !self.closed => {
                self.closed = true;
                Err("The extension supervisor disconnected.".into())
            }
            _ => Ok(None),
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.input = None;
        let deadline = Instant::now() + Duration::from_secs(4);
        // Continue draining so a full output pipe cannot prevent cooperative teardown.
        while Instant::now() < deadline {
            for _ in 0..64 {
                if self.output.try_recv().is_err() {
                    break;
                }
            }
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[tauri::command]
pub async fn extension_exchange(
    app: tauri::AppHandle,
    operation: Operation,
) -> Result<Snapshot, String> {
    if crate::settings::is_child() {
        return Err("Extension commands belong to the launcher.".into());
    }
    let panel = app
        .try_state::<crate::panel::Panel>()
        .ok_or("Extensions require the resident launcher.")?;
    let _lease = panel.work.begin();
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<crate::panel::Panel>().extension(operation)
    })
    .await
    .map_err(|e| e.to_string())?
}
