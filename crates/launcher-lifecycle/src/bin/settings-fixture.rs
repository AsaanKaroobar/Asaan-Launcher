//! Test-only settings peer. Uses the same request IDs as the panel on its own private pipe.
use asaaan_lifecycle::{Packet, Request, wire};
use asaaan_personalization::{Operation, Snapshot};
fn exchange(
    input: &mut impl std::io::BufRead,
    output: &mut impl std::io::Write,
    operation: Operation,
) -> Snapshot {
    wire::write(
        output,
        &Packet::PersonalizationRequest {
            request_id: 1,
            operation,
        },
    )
    .unwrap();
    match wire::read(input).unwrap().unwrap() {
        Packet::PersonalizationReply {
            request_id: 1,
            snapshot: Some(snapshot),
            error: None,
        } => *snapshot,
        other => panic!("Unexpected reply: {other:?}"),
    }
}
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
                let before = exchange(&mut input, &mut output, Operation::Read);
                assert_eq!(before.records["app:one"].count, 1);
                std::thread::sleep(std::time::Duration::from_millis(20));
                exchange(
                    &mut input,
                    &mut output,
                    Operation::Alias {
                        id: "app:one".into(),
                        alias: "configured".into(),
                    },
                );
                exchange(
                    &mut input,
                    &mut output,
                    Operation::RememberSearches { enabled: false },
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
