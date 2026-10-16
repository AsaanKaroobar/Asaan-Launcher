//! A worker owns the child; pipe writes never block Tauri's UI thread.
use crate::{Effect, Lifetime, Packet, Request, wire};
use std::{
    io::BufReader,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Sender},
    thread,
    time::{Duration, Instant},
};
#[derive(Debug, Clone)]
pub enum Event {
    Started(u32),
    Exited(u32),
    Packet(Packet),
    Failed(String),
}
enum Input {
    Request(Request),
    Packet(u64, Packet),
    Broken(u64),
    Stop,
}
#[derive(Clone)]
pub struct Controller(Sender<Input>);
impl Controller {
    pub fn request(&self, request: Request) {
        let _ = self.0.send(Input::Request(request));
    }
    pub fn stop(&self) {
        let _ = self.0.send(Input::Stop);
    }
}
pub struct Config {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub readiness_timeout: Duration,
    pub close_timeout: Duration,
}
impl Config {
    pub fn panel(executable: PathBuf) -> Self {
        Self {
            executable,
            arguments: vec!["--panel".into()],
            readiness_timeout: Duration::from_secs(10),
            close_timeout: Duration::from_secs(2),
        }
    }
}
struct Running {
    child: Child,
    writer: Sender<Packet>,
    started: Instant,
    closing: Option<Instant>,
}
pub fn start(
    config: Config,
    mut observe: impl FnMut(Event) + Send + 'static,
) -> (Controller, thread::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let control = Controller(tx.clone());
    let worker = thread::spawn(move || {
        let mut lifetime = Lifetime::default();
        let mut running: Option<Running> = None;
        let mut generation = 0u64;
        loop {
            let mut effects = match rx.recv_timeout(Duration::from_millis(20)) {
                Ok(Input::Request(r)) => lifetime.request(r),
                Ok(Input::Packet(g, packet)) if g == generation => {
                    let effects = match &packet {
                        Packet::Ready => lifetime.ready(),
                        Packet::Idle { sequence } => lifetime.idle(*sequence),
                        _ => vec![],
                    };
                    observe(Event::Packet(packet));
                    effects
                }
                Ok(Input::Broken(g)) if g == generation => {
                    if let Some(r) = &mut running {
                        let _ = r.child.kill();
                    }
                    vec![]
                }
                Ok(Input::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    if let Some(mut r) = running.take() {
                        let _ = r.writer.send(Packet::Shutdown);
                        drop(r.writer);
                        let deadline = Instant::now() + config.close_timeout;
                        while r.child.try_wait().ok().flatten().is_none()
                            && Instant::now() < deadline
                        {
                            thread::sleep(Duration::from_millis(10));
                        }
                        let _ = r.child.kill();
                        let _ = r.child.wait();
                    }
                    break;
                }
                _ => vec![],
            };
            if let Some(r) = &mut running {
                match r.child.try_wait() {
                    Ok(Some(_)) => {
                        observe(Event::Exited(r.child.id()));
                        running = None;
                        effects.extend(lifetime.exited());
                    }
                    Err(error) => {
                        observe(Event::Failed(error.to_string()));
                        let _ = r.child.kill();
                        let _ = r.child.wait();
                        running = None;
                        effects.extend(lifetime.exited());
                    }
                    Ok(None) => {
                        if lifetime.awaiting_readiness()
                            && r.started.elapsed() >= config.readiness_timeout
                        {
                            effects.extend(lifetime.readiness_timed_out());
                        }
                        if r.closing
                            .is_some_and(|at| at.elapsed() >= config.close_timeout)
                        {
                            let _ = r.child.kill();
                        }
                    }
                }
            }
            let mut queue: std::collections::VecDeque<_> = effects.into();
            while let Some(effect) = queue.pop_front() {
                match effect {
                    Effect::Start => {
                        generation += 1;
                        match Command::new(&config.executable)
                            .args(&config.arguments)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::inherit())
                            .spawn()
                        {
                            Ok(mut child) => {
                                let mut input =
                                    BufReader::new(child.stdout.take().expect("piped stdout"));
                                let mut output = child.stdin.take().expect("piped stdin");
                                let reader_tx = tx.clone();
                                let writer_tx = tx.clone();
                                let g = generation;
                                thread::spawn(move || {
                                    while let Ok(Some(packet)) = wire::read(&mut input) {
                                        if reader_tx.send(Input::Packet(g, packet)).is_err() {
                                            return;
                                        }
                                    }
                                    let _ = reader_tx.send(Input::Broken(g));
                                });
                                let (writer, packets) = mpsc::channel();
                                thread::spawn(move || {
                                    for packet in packets {
                                        if wire::write(&mut output, &packet).is_err() {
                                            let _ = writer_tx.send(Input::Broken(g));
                                            break;
                                        }
                                    }
                                });
                                observe(Event::Started(child.id()));
                                running = Some(Running {
                                    child,
                                    writer,
                                    started: Instant::now(),
                                    closing: None,
                                });
                            }
                            Err(error) => {
                                observe(Event::Failed(error.to_string()));
                                queue.extend(lifetime.exited());
                            }
                        }
                    }
                    Effect::Terminate => {
                        if let Some(r) = &mut running {
                            let _ = r.child.kill();
                        }
                    }
                    Effect::Send(packet) => {
                        if let Some(r) = &mut running {
                            if packet == Packet::Shutdown {
                                r.closing = Some(Instant::now());
                            }
                            if r.writer.send(packet).is_err() {
                                let _ = r.child.kill();
                            }
                        }
                    }
                }
            }
        }
    });
    (control, worker)
}
