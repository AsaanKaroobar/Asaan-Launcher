//! One Bun process per command. All IO uses private pipes; no WebView or shell is involved.
use crate::{
    manifest::{Command as ExtensionCommand, safe_name},
    protocol::{Decoder, Message},
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, SyncSender},
    time::{Duration, Instant},
};

enum Chunk {
    Out(Vec<u8>),
    Err(Vec<u8>),
    Closed,
    Failed,
}
#[derive(Debug)]
pub enum Event {
    Message(Message),
    Ended(ExitStatus),
}
/// Floe's three-second timer calls this; an unanswered ping is unresponsive strictly after eight seconds.
#[derive(Default)]
pub struct Heartbeat {
    sent: Option<Duration>,
    pub unresponsive: bool,
}
impl Heartbeat {
    pub fn tick(&mut self, now: Duration) -> bool {
        if let Some(sent) = self.sent {
            if now.saturating_sub(sent) > Duration::from_secs(8) {
                self.unresponsive = true;
            }
            false
        } else {
            self.sent = Some(now);
            true
        }
    }
    pub fn pong(&mut self) {
        self.sent = None;
        self.unresponsive = false;
    }
}

pub struct Session {
    child: Child,
    input: Option<SyncSender<Vec<u8>>>,
    chunks: Receiver<Chunk>,
    decoder: Decoder,
    pending: VecDeque<Message>,
    pub log: String,
    closed: usize,
    stopped: bool,
    recover: bool,
    #[cfg(windows)]
    job: Job,
}
impl Session {
    pub fn spawn(
        bun: &Path,
        host: &Path,
        command: &ExtensionCommand,
        support: &Path,
        arguments: &Value,
    ) -> Result<Self, String> {
        if !safe_name(&command.extension_name)
            || !safe_name(&command.name)
            || !arguments.is_object()
        {
            return Err("Invalid extension command or arguments".into());
        }
        let arguments = serde_json::to_string(arguments).map_err(|e| e.to_string())?;
        if arguments.len() > 65536 {
            return Err("Extension arguments exceed 64 KiB".into());
        }
        let mut process = Command::new(bun);
        process
            .args(["--smol"])
            .arg(host)
            .arg(&command.extension_dir)
            .arg(&command.name)
            .arg(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("ASAAAN_EXTENSION_DATA", support)
            .env("FLOE_LAUNCH_TYPE", "userInitiated")
            .env("FLOE_ACCESS", "off")
            .env("FLOE_APPEARANCE", "dark")
            .env("FLOE_RENDER_REFERENCES", "1")
            .env_remove("FLOE_OAUTH")
            .env_remove("FLOE_PREFERENCES")
            .env_remove("FLOE_AI")
            .env_remove("FLOE_DEVELOPMENT")
            .env_remove("BUN_OPTIONS")
            .env_remove("NODE_OPTIONS")
            .env_remove("NODE_PATH");
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            process.process_group(0);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            process.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let mut child = process
            .spawn()
            .map_err(|e| format!("The extension couldn't start: {e}"))?;
        #[cfg(windows)]
        let job = match Job::attach(&child) {
            Ok(job) => job,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        };
        let mut stdin = child.stdin.take().ok_or("Extension input unavailable")?;
        let stdout = child.stdout.take().ok_or("Extension output unavailable")?;
        let stderr = child
            .stderr
            .take()
            .ok_or("Extension error pipe unavailable")?;
        let (input, writes) = mpsc::sync_channel::<Vec<u8>>(16);
        std::thread::spawn(move || {
            while let Ok(bytes) = writes.recv() {
                if stdin.write_all(&bytes).is_err() {
                    break;
                }
            }
        });
        let (tx, chunks) = mpsc::sync_channel(16);
        fn read(mut pipe: impl Read + Send + 'static, tx: SyncSender<Chunk>, err: bool) {
            std::thread::spawn(move || {
                let mut buffer = [0; 8192];
                loop {
                    match pipe.read(&mut buffer) {
                        Ok(0) => {
                            let _ = tx.send(Chunk::Closed);
                            break;
                        }
                        Ok(n) => {
                            if tx
                                .send(if err {
                                    Chunk::Err(buffer[..n].to_vec())
                                } else {
                                    Chunk::Out(buffer[..n].to_vec())
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => {
                            let _ = tx.send(Chunk::Failed);
                            break;
                        }
                    }
                }
            });
        }
        read(stdout, tx.clone(), false);
        read(stderr, tx, true);
        Ok(Self {
            child,
            input: Some(input),
            chunks,
            decoder: Decoder::default(),
            pending: VecDeque::new(),
            log: String::new(),
            closed: 0,
            stopped: false,
            recover: true,
            #[cfg(windows)]
            job,
        })
    }
    pub fn set_recovery(&mut self, enabled: bool) {
        self.recover = enabled;
    }
    pub fn pid(&self) -> u32 {
        self.child.id()
    }
    pub fn send(&self, message: &Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec(message).map_err(|e| e.to_string())?;
        if bytes.len() > 65536 {
            return Err("Extension input exceeds 64 KiB".into());
        }
        bytes.push(b'\n');
        self.input
            .as_ref()
            .ok_or("Extension input is closed")?
            .try_send(bytes)
            .map_err(|_| "Extension input is closed or busy".into())
    }
    pub fn close_input(&mut self) {
        self.input = None;
    }
    /// Apply each event before calling receive again: chunks cannot overtake a suspended application.
    pub fn receive(&mut self, timeout: Duration) -> Result<Option<Event>, String> {
        self.receive_forward(timeout, |_| Ok(()))
    }
    pub fn receive_forward(
        &mut self,
        timeout: Duration,
        mut forward: impl FnMut(&[u8]) -> Result<(), String>,
    ) -> Result<Option<Event>, String> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(message) = self.pending.pop_front() {
                if self.recover && matches!(message, Message::UnresolvedRender) {
                    self.send(&json!({"type":"fullRender"}))?;
                }
                return Ok(Some(Event::Message(message)));
            }
            if self.closed >= 2
                && let Some(status) = self.child.try_wait().map_err(|e| e.to_string())?
            {
                return Ok(Some(Event::Ended(status)));
            }
            let Some(left) = deadline.checked_duration_since(Instant::now()) else {
                return Ok(None);
            };
            match self
                .chunks
                .recv_timeout(left.min(Duration::from_millis(25)))
            {
                Ok(Chunk::Out(bytes)) => {
                    forward(&bytes)?;
                    self.pending.extend(
                        self.decoder
                            .append(&bytes)
                            .map_err(|e| format!("Invalid extension protocol: {e:?}"))?,
                    );
                }
                Ok(Chunk::Err(bytes)) => {
                    self.log.push_str(&String::from_utf8_lossy(&bytes));
                    if self.log.len() > 20000 {
                        let mut start = self.log.len() - 16000;
                        while !self.log.is_char_boundary(start) {
                            start += 1;
                        }
                        self.log.drain(..start);
                    }
                }
                Ok(Chunk::Closed) => self.closed += 1,
                Ok(Chunk::Failed) => return Err("Extension pipe failed".into()),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    if let Some(status) = self.child.try_wait().map_err(|e| e.to_string())? {
                        return Ok(Some(Event::Ended(status)));
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
    }
    /// This foundation deliberately exposes no native API operations. Every request with an id is answered.
    pub fn refuse_request(&self, fields: &serde_json::Map<String, Value>) -> Result<(), String> {
        if fields.get("type").and_then(Value::as_str) == Some("request")
            && let Some(id) = fields.get("id").and_then(Value::as_i64)
        {
            self.send(&json!({"type":"reply","id":id,"error":"This native extension API is unavailable in this checkpoint."}))?;
        }
        Ok(())
    }
    pub fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        self.close_input();
        #[cfg(unix)]
        {
            let group = -(self.child.id() as i32);
            // This is the new, owned host process group, never a discovered application's PID.
            unsafe {
                libc::kill(group, libc::SIGTERM);
            }
            let deadline = Instant::now() + Duration::from_secs(1);
            while Instant::now() < deadline {
                let _ = self.child.try_wait();
                if unsafe { libc::kill(group, 0) } != 0 {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            unsafe {
                libc::kill(group, libc::SIGKILL);
            }
        }
        #[cfg(windows)]
        {
            let deadline = Instant::now() + Duration::from_secs(1);
            while Instant::now() < deadline {
                if matches!(self.child.try_wait(), Ok(Some(_))) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            self.job.terminate();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
struct Job(windows::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl Job {
    fn attach(child: &Child) -> Result<Self, String> {
        use std::{mem::size_of, os::windows::io::AsRawHandle};
        use windows::Win32::{
            Foundation::{CloseHandle, HANDLE},
            System::JobObjects::*,
        };
        unsafe {
            let handle = CreateJobObjectW(None, None).map_err(|e| e.to_string())?;
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if let Err(e) = SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .and_then(|_| AssignProcessToJobObject(handle, HANDLE(child.as_raw_handle())))
            {
                let _ = CloseHandle(handle);
                return Err(e.to_string());
            }
            Ok(Self(handle))
        }
    }
    fn terminate(&self) {
        unsafe {
            let _ = windows::Win32::System::JobObjects::TerminateJobObject(self.0, 1);
        }
    }
}
#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heartbeat_uses_strict_eight_second_boundary_and_recovers_on_pong() {
        let mut h = Heartbeat::default();
        assert!(h.tick(Duration::ZERO));
        assert!(!h.tick(Duration::from_secs(8)));
        assert!(!h.unresponsive);
        assert!(!h.tick(Duration::from_millis(8001)));
        assert!(h.unresponsive);
        h.pong();
        assert!(!h.unresponsive);
        assert!(h.tick(Duration::from_secs(9)));
    }
}
