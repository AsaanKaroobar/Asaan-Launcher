//! Test-only foreground. It has no disk path and can only forward typed operations.
use asaaan_lifecycle::{Packet, Request, wire};
use asaaan_personalization::Operation;
use std::io::BufReader;
fn exchange(
    input: &mut impl std::io::BufRead,
    output: &mut impl std::io::Write,
    id: u64,
    operation: Operation,
) -> asaaan_personalization::Snapshot {
    wire::write(
        output,
        &Packet::PersonalizationRequest {
            request_id: id,
            operation,
        },
    )
    .unwrap();
    match wire::read(input).unwrap().unwrap() {
        Packet::PersonalizationReply {
            request_id,
            snapshot: Some(snapshot),
            error: None,
        } if request_id == id => *snapshot,
        other => panic!("Unexpected resident response: {other:?}"),
    }
}
fn main() {
    let mut input = BufReader::new(std::io::stdin());
    let mut output = std::io::stdout();
    wire::write(&mut output, &Packet::Ready).unwrap();
    while let Ok(Some(packet)) = wire::read(&mut input) {
        match packet {
            Packet::Request {
                request: Request::Show,
                ..
            } => {
                let before = exchange(&mut input, &mut output, 1, Operation::Read);
                assert_eq!(before.records["app:one"].count, 1);
                let next = exchange(
                    &mut input,
                    &mut output,
                    2,
                    Operation::RecordLaunch {
                        id: "app:two".into(),
                        query: "two".into(),
                    },
                );
                assert_eq!(next.records["app:one"].count, 1);
                assert_eq!(
                    next.records["app:two"].count,
                    before.records.get("app:two").map_or(1, |r| r.count + 1)
                );
                let next = exchange(
                    &mut input,
                    &mut output,
                    3,
                    Operation::Favorite {
                        id: "app:two".into(),
                        enabled: true,
                    },
                );
                assert!(next.favorites.contains(&"app:two".into()));
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
