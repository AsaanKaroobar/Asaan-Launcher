// Process equivalents of Floe PanelProcessTests.swift and privatePipePreservesRequestOrdering.
// Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
#![allow(non_snake_case)]
use asaaan_lifecycle::{
    Packet, Request,
    process::{self, Config, Event},
};
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
fn config(mode: &str) -> Config {
    Config {
        executable: PathBuf::from(env!("CARGO_BIN_EXE_lifecycle-fixture")),
        arguments: vec![mode.into()],
        readiness_timeout: Duration::from_millis(300),
        close_timeout: Duration::from_millis(100),
    }
}
struct Fixture {
    control: process::Controller,
    worker: Option<std::thread::JoinHandle<()>>,
    events: mpsc::Receiver<Event>,
}
impl Fixture {
    fn new(config: Config) -> Self {
        let (tx, events) = mpsc::channel();
        let (control, worker) = process::start(config, move |e| {
            let _ = tx.send(e);
        });
        Self {
            control,
            worker: Some(worker),
            events,
        }
    }
    fn until(&self, mut check: impl FnMut(Event) -> bool) {
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(Instant::now() < end, "process event deadline");
            if let Ok(e) = self.events.recv_timeout(Duration::from_millis(100))
                && check(e)
            {
                break;
            }
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.control.stop();
        self.worker.take().unwrap().join().unwrap();
    }
}
#[test]
fn repeatedPreReadyCrashesStopSpawning() {
    let f = Fixture::new(config("crash"));
    f.control.request(Request::Show);
    let mut starts = 0;
    let mut exits = 0;
    f.until(|e| {
        match e {
            Event::Started(_) => starts += 1,
            Event::Exited(_) => exits += 1,
            _ => {}
        }
        exits == 3
    });
    assert_eq!(starts, 3);
    assert!(f.events.recv_timeout(Duration::from_millis(400)).is_err());
}
#[test]
fn launchErrorsStopAfterThreeAttemptsAndAllowANewRequest() {
    let mut c = config("echo");
    c.executable = PathBuf::from("/missing-asaaan-checkpoint-fixture");
    let f = Fixture::new(c);
    for _ in 0..2 {
        f.control.request(Request::Show);
        let mut failures = 0;
        f.until(|e| {
            if matches!(e, Event::Failed(_)) {
                failures += 1;
            }
            failures == 3
        });
        assert!(f.events.recv_timeout(Duration::from_millis(100)).is_err());
    }
}
fn surviving(mode: &str) {
    let dir = std::env::temp_dir().join(format!("asaaan-lifecycle-{}-{mode}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let counter = dir.join("attempts");
    let _ = std::fs::remove_file(&counter);
    let mut c = config(mode);
    c.arguments.push(counter.to_string_lossy().into_owned());
    let f = Fixture::new(c);
    f.control.request(Request::Show);
    f.control.request(Request::Toggle);
    let mut received = vec![];
    f.until(|e| {
        if let Event::Packet(Packet::Request { request, sequence }) = e {
            received.push((request, sequence));
        }
        received.len() == 2
    });
    assert_eq!(received, vec![(Request::Show, 1), (Request::Toggle, 2)]);
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "2");
    drop(f);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn aRequestSurvivesALaunchError() {
    let dir = std::env::temp_dir().join(format!("asaaan-launch-error-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let executable = dir.join(if cfg!(windows) {
        "fixture.exe"
    } else {
        "fixture"
    });
    let _ = std::fs::remove_file(&executable);
    let mut c = config("echo");
    c.executable = executable.clone();
    let (tx, rx) = mpsc::channel();
    let destination = executable.clone();
    let staged = dir.join("staged-fixture");
    std::fs::copy(env!("CARGO_BIN_EXE_lifecycle-fixture"), &staged).unwrap();
    let (control, worker) = process::start(c, move |event| {
        if let Event::Failed(ref error) = event {
            eprintln!("Launch-error fixture: {error}");
            std::fs::rename(&staged, &destination).unwrap();
        }
        tx.send(event).unwrap();
    });
    control.request(Request::Show);
    let end = Instant::now() + Duration::from_secs(10);
    let mut failures = 0;
    loop {
        assert!(Instant::now() < end);
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            match event {
                Event::Failed(_) => failures += 1,
                Event::Packet(Packet::Request {
                    request: Request::Show,
                    sequence: 1,
                }) => break,
                _ => {}
            }
        }
    }
    assert_eq!(failures, 1);
    control.stop();
    worker.join().unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn aRequestSurvivesTheFirstChildsReadinessTimeout() {
    surviving("hang-first");
}
#[test]
fn repeatedReadinessTimeoutsStopSpawningEvenWhenGracefulShutdownIsIgnored() {
    let f = Fixture::new(config("hang"));
    f.control.request(Request::Show);
    let mut starts = 0;
    let mut exits = 0;
    f.until(|e| {
        match e {
            Event::Started(_) => starts += 1,
            Event::Exited(_) => exits += 1,
            _ => {}
        }
        exits == 3
    });
    assert_eq!(starts, 3);
    assert!(f.events.recv_timeout(Duration::from_millis(400)).is_err());
}
#[test]
fn privatePipePreservesRequestOrderingAndReopensAfterIdle() {
    let f = Fixture::new(config("echo"));
    let mut pids = vec![];
    for _ in 0..3 {
        f.control.request(Request::Show);
        f.until(|e| {
            if let Event::Started(pid) = e {
                pids.push(pid);
            }
            matches!(
                e,
                Event::Packet(Packet::Request {
                    request: Request::Show,
                    ..
                })
            )
        });
        f.control.request(Request::Hide);
        f.until(|e| matches!(e, Event::Exited(_)));
    }
    assert_eq!(pids.len(), 3);
    assert!(pids.windows(2).all(|p| p[0] != p[1]));
}
#[test]
fn pipeEofExitsTheChild() {
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_lifecycle-fixture"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    drop(child.stdin.take());
    let end = Instant::now() + Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() > end {
            child.kill().unwrap();
            panic!("child did not exit on EOF");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
