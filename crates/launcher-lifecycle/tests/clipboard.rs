use asaaan_lifecycle::{
    Packet, Request, clipboard,
    process::{self, Config, Event},
    wire,
};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
#[test]
fn resident_acknowledges_copy_and_keeps_text_after_foreground_restarts() {
    let text = Arc::new(Mutex::new(String::new()));
    let service = text.clone();
    let (tx, rx) = mpsc::channel();
    let (control, worker) = process::start_serviced(
        Config {
            executable: env!("CARGO_BIN_EXE_clipboard-fixture").into(),
            arguments: vec![],
            readiness_timeout: Duration::from_secs(3),
            close_timeout: Duration::from_secs(1),
        },
        move |e| {
            let _ = tx.send(e);
        },
        move |packet| {
            if let Packet::ClipboardWrite {
                request_id,
                text,
                deadline,
            } = packet
            {
                let result = clipboard::validate(text, *deadline, clipboard::now());
                if result.is_ok() {
                    *service.lock().unwrap() = text.clone();
                }
                Some(Packet::ClipboardWritten {
                    request_id: *request_id,
                    error: result.err(),
                })
            } else {
                None
            }
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
        assert_eq!(*text.lock().unwrap(), "Native 日本語 ; path\n");
    }
    control.stop();
    worker.join().unwrap();
}
#[test]
fn private_clipboard_wire_preserves_utf8_and_rejects_forged_fields() {
    let packet = Packet::ClipboardWrite {
        request_id: 7,
        text: "日本語\n; $literal".into(),
        deadline: 123,
    };
    let mut bytes = vec![];
    wire::write(&mut bytes, &packet).unwrap();
    assert_eq!(
        wire::read(&mut std::io::Cursor::new(bytes)).unwrap(),
        Some(packet)
    );
    assert!(
        serde_json::from_str::<Packet>(
            r#"{"type":"clipboardWrite","request_id":7,"text":"x","deadline":123,"command":"bad"}"#
        )
        .is_err()
    );
}
