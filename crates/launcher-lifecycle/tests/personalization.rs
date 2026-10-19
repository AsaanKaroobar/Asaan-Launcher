use asaaan_lifecycle::{
    Packet, Request,
    process::{self, Config, Event},
};
use asaaan_personalization::{Operation, Store};
use std::{
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};
#[test]
fn foreground_restarts_cannot_overwrite_the_resident_with_stale_snapshots() {
    let dir = std::env::temp_dir().join(format!(
        "asaaan-personalization-process-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("state.json");
    let mut initial = Store::open(&path).unwrap();
    initial
        .execute(
            &Operation::RecordLaunch {
                id: "app:one".into(),
                query: "one".into(),
            },
            100,
        )
        .unwrap();
    let store = Arc::new(Mutex::new(initial));
    let service = store.clone();
    let (tx, rx) = mpsc::channel();
    let config = Config {
        executable: env!("CARGO_BIN_EXE_personalization-fixture").into(),
        arguments: vec![],
        readiness_timeout: Duration::from_secs(3),
        close_timeout: Duration::from_secs(1),
    };
    let (control, worker) = process::start_serviced(
        config,
        move |event| {
            let _ = tx.send(event);
        },
        move |packet| {
            let Packet::PersonalizationRequest {
                request_id,
                operation,
            } = packet
            else {
                return None;
            };
            let snapshot = service.lock().unwrap().execute(operation, 200).unwrap();
            Some(Packet::PersonalizationReply {
                request_id: *request_id,
                snapshot: Some(Box::new(snapshot)),
                error: None,
            })
        },
    );
    for _ in 0..3 {
        control.request(Request::Show);
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(Instant::now() < end);
            if matches!(
                rx.recv_timeout(Duration::from_millis(50)),
                Ok(Event::Packet(Packet::SearchVerified))
            ) {
                break;
            }
        }
        control.request(Request::Hide);
        loop {
            assert!(Instant::now() < end);
            if matches!(
                rx.recv_timeout(Duration::from_millis(50)),
                Ok(Event::Exited(_))
            ) {
                break;
            }
        }
    }
    control.stop();
    worker.join().unwrap();
    let disk = Store::open(&path).unwrap().snapshot();
    assert_eq!(disk.records["app:one"].count, 1);
    assert_eq!(disk.records["app:two"].count, 3);
    assert_eq!(disk.queries, ["one", "two"]);
    assert_eq!(disk.favorites, ["app:two"]);
    assert_eq!(disk, store.lock().unwrap().snapshot());
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}
