//! Test-only foreground; copies survive this process exiting.
use asaaan_lifecycle::{Packet, Request, clipboard, wire};
fn main() {
    let mut input = std::io::BufReader::new(std::io::stdin());
    let mut output = std::io::stdout();
    wire::write(&mut output, &Packet::Ready).unwrap();
    while let Ok(Some(packet)) = wire::read(&mut input) {
        match packet {
            Packet::Request {
                request: Request::Show,
                ..
            } => {
                wire::write(
                    &mut output,
                    &Packet::ClipboardWrite {
                        request_id: 71,
                        text: "Native 日本語 ; path\n".into(),
                        deadline: clipboard::now() + clipboard::TIMEOUT_MS,
                    },
                )
                .unwrap();
                assert_eq!(
                    wire::read(&mut input).unwrap(),
                    Some(Packet::ClipboardWritten {
                        request_id: 71,
                        error: None
                    })
                );
                wire::write(&mut output, &Packet::SearchVerified).unwrap();
            }
            Packet::Request {
                request: Request::Hide,
                sequence,
            } => wire::write(&mut output, &Packet::Idle { sequence }).unwrap(),
            Packet::Shutdown => break,
            _ => {}
        }
    }
}
