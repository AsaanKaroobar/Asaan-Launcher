//! Bundled diagnostic and private supervisor entry points; neither creates a WebView.
use asaaan_extensions::{
    manifest::{self, Platform},
    protocol::Message,
    session::{Event, Heartbeat, Session},
};
use serde_json::{Value, json};
use std::{
    io::{BufRead, Read, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

static CANCELLED: AtomicBool = AtomicBool::new(false);
#[cfg(unix)]
extern "C" fn cancel(_: i32) {
    CANCELLED.store(true, Ordering::Relaxed);
}
pub(crate) fn resources() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let parent = exe.parent().ok_or("Application path unavailable")?;
    let base = if cfg!(target_os = "linux") {
        parent.join("../lib/Asaaan Launcher/extension-runtime")
    } else if cfg!(target_os = "macos") {
        parent.join("../Resources/extension-runtime")
    } else {
        parent.join("extension-runtime")
    };
    #[cfg(debug_assertions)]
    if !base.join("runtime/host.ts").is_file() {
        let development =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.runtime-bundle");
        if development.join("runtime/host.ts").is_file() {
            return Ok(development);
        }
    }
    if base.join("runtime/host.ts").is_file() {
        Ok(base)
    } else {
        Err("Bundled extension runtime is missing. Reinstall this checkpoint.".into())
    }
}
fn output(v: &Value) -> Result<(), String> {
    let mut out = std::io::stdout().lock();
    serde_json::to_writer(&mut out, v).map_err(|e| e.to_string())?;
    out.write_all(b"\n")
        .and_then(|_| out.flush())
        .map_err(|e| e.to_string())
}
pub fn start() -> Option<i32> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let worker = args.first().is_some_and(|a| a == "--extension-worker");
    if !worker && !args.iter().any(|a| a == "--extension-check") {
        return None;
    }
    if args.len() != 3
        || !worker && args[0] != "--extension-check"
        || !["hello", "diagnostics", "host-probe", "list-lab"].contains(&args[1].as_str())
        || !manifest::safe_name(&args[2])
    {
        eprintln!(
            "Usage: asaaan-launcher --extension-check hello|diagnostics|host-probe COMMAND (exclusive mode; NDJSON input)"
        );
        return Some(64);
    }
    Some(match check(&args[1], &args[2], worker) {
        Ok(()) => 0,
        Err(error) => {
            if worker {
                let _ = output(&json!({"type":"error","fatal":true,"message":error}));
            }
            eprintln!("{error}");
            69
        }
    })
}
fn check(extension: &str, name: &str, worker: bool) -> Result<(), String> {
    let root = resources()?;
    let commands = manifest::scan(&root.join("extensions"), Platform::current());
    let command = commands
        .iter()
        .find(|c| c.extension_name == extension && c.name == name)
        .ok_or("Bundled command is unavailable on this platform")?;
    let support = crate::data::personalization_path()?
        .parent()
        .ok_or("Application data unavailable")?
        .join("extensions-data");
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGTERM, cancel as *const () as usize);
        libc::signal(libc::SIGINT, cancel as *const () as usize);
    }
    let mut host = Session::spawn(
        &root.join(if cfg!(windows) { "bun.exe" } else { "bun" }),
        &root.join("runtime/host.ts"),
        command,
        &support,
        &json!({}),
    )?;
    if worker {
        host.set_recovery(false);
    }
    output(&json!({"type":"started","pid":host.pid(),"command":command.id,"checkpoint":"CP014"}))?;
    let (tx, rx) = mpsc::sync_channel(16);
    std::thread::spawn(move || {
        let mut input = std::io::BufReader::new(std::io::stdin());
        loop {
            let mut line = Vec::new();
            match input.by_ref().take(65538).read_until(b'\n', &mut line) {
                Ok(0) => break,
                Ok(_) if line.len() <= 65537 && line.last() == Some(&b'\n') => {
                    if let Ok(value) = serde_json::from_slice::<Value>(&line)
                        && tx.send(Ok(value)).is_err()
                    {
                        break;
                    }
                }
                _ => {
                    let _ = tx.send(Err("Invalid or oversized extension-check input"));
                    break;
                }
            }
        }
    });
    let started = Instant::now();
    let mut relay = Vec::new();
    let mut heartbeat = Heartbeat::default();
    let mut next_tick = Duration::from_secs(3);
    let mut eof = None;
    loop {
        if CANCELLED.load(Ordering::Relaxed) {
            host.stop();
            output(&json!({"type":"stopped","reason":"cancelled"}))?;
            return Ok(());
        }
        if eof.is_none() {
            loop {
                match rx.try_recv() {
                    Ok(Ok(v)) => host.send(&v)?,
                    Ok(Err(e)) => return Err(e.into()),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        host.close_input();
                        eof = Some(Instant::now());
                        break;
                    }
                }
            }
        }
        if eof.is_some_and(|at: Instant| at.elapsed() > Duration::from_secs(1)) {
            host.stop();
            output(&json!({"type":"stopped","reason":"inputClosed"}))?;
            return Ok(());
        }
        if started.elapsed() >= next_tick && eof.is_none() {
            if heartbeat.tick(started.elapsed()) {
                host.send(&json!({"type":"ping"}))?;
            }
            next_tick += Duration::from_secs(3);
            if heartbeat.unresponsive {
                output(&json!({"type":"unresponsive"}))?;
            }
        }
        if !worker && started.elapsed() > Duration::from_secs(25) {
            return Err("Extension check reached its 25-second diagnostic deadline".into());
        }
        match host.receive_forward(Duration::from_millis(25), |bytes| {
            if worker {
                relay.extend_from_slice(bytes);
                if let Some(end) = relay.iter().rposition(|b| *b == b'\n') {
                    let mut out = std::io::stdout().lock();
                    out.write_all(&relay[..=end])
                        .and_then(|_| out.flush())
                        .map_err(|e| e.to_string())?;
                    relay.drain(..=end);
                }
            }
            Ok(())
        })? {
            Some(Event::Message(message)) => match message {
                Message::Render(tree) if !worker => {
                    output(&json!({"type":"render","tree":tree.map(|n|n.json())}))?
                }
                Message::UnresolvedRender if !worker => {
                    output(&json!({"type":"unresolvedRender"}))?
                }
                Message::Fields(fields) => {
                    if fields.get("type").and_then(Value::as_str) == Some("pong") {
                        heartbeat.pong();
                    }
                    if !worker {
                        host.refuse_request(&fields)?;
                        output(&Value::Object(fields))?;
                    }
                }
                _ => {}
            },
            Some(Event::Ended(status)) => {
                output(
                    &json!({"type":"ended","success":status.success(),"code":status.code(),"logBytes":host.log.len()}),
                )?;
                if !host.log.is_empty() {
                    eprint!("{}", host.log);
                }
                return Ok(());
            }
            None => {}
        }
    }
}
