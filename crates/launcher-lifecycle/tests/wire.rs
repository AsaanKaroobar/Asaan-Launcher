#![allow(non_snake_case)]
use asaaan_lifecycle::{Packet, wire};
use std::io::{BufReader, Cursor};
#[test]
fn fragmentedPacketsRetainOrder() {
    let bytes = b"{\"type\":\"ready\"}\n{\"type\":\"idle\",\"sequence\":7}\n";
    let mut r = BufReader::with_capacity(1, Cursor::new(bytes));
    assert_eq!(wire::read(&mut r).unwrap(), Some(Packet::Ready));
    assert_eq!(
        wire::read(&mut r).unwrap(),
        Some(Packet::Idle { sequence: 7 })
    );
    assert_eq!(wire::read(&mut r).unwrap(), None);
}
#[test]
fn rejectsTruncatedMalformedAndUnknownPackets() {
    for s in [
        "{",
        "garbage\n",
        "{\"type\":\"quitEverything\"}\n",
        "{\"type\":\"idle\",\"sequence\":-1}\n",
    ] {
        assert!(wire::read(&mut Cursor::new(s)).is_err());
    }
}
#[test]
fn rejectsOversizedInputBeforeAllocationGrowsUnbounded() {
    let bytes = vec![b'x'; wire::MAX_PACKET + 1];
    assert!(wire::read(&mut Cursor::new(bytes)).is_err());
}
#[test]
fn roundTripUsesOneLinePerTypedPacket() {
    let mut out = vec![];
    wire::write(&mut out, &Packet::Idle { sequence: 42 }).unwrap();
    assert_eq!(out.iter().filter(|b| **b == b'\n').count(), 1);
    assert_eq!(
        wire::read(&mut Cursor::new(out)).unwrap(),
        Some(Packet::Idle { sequence: 42 })
    );
}

#[test]
fn extensionCommandsAndAcknowledgementsRoundTripWithUnicodeAndNoRawHandlers() {
    use asaaan_extensions::view::{Operation, Snapshot};
    for packet in [
        Packet::ExtensionRequest {
            request_id: 7,
            operation: Operation::Start {
                id: "command:list-lab/planets".into(),
                query: "planètes ✓".into(),
            },
        },
        Packet::ExtensionRequest {
            request_id: 8,
            operation: Operation::Run {
                token: 4,
                id: 9,
                revision: 12,
                query: "火星".into(),
            },
        },
        Packet::ExtensionReply {
            request_id: 8,
            snapshot: Some(Box::new(Snapshot::default())),
            error: None,
        },
        Packet::ExtensionReply {
            request_id: 9,
            snapshot: None,
            error: Some("Session replaced".into()),
        },
    ] {
        let mut bytes = vec![];
        wire::write(&mut bytes, &packet).unwrap();
        assert_eq!(bytes.iter().filter(|b| **b == b'\n').count(), 1);
        let mut input = BufReader::with_capacity(1, Cursor::new(bytes));
        assert_eq!(wire::read(&mut input).unwrap(), Some(packet));
    }
}
#[test]
fn extensionWireRefusesArbitraryEventPropsAndPaths() {
    for text in [
        "{\"type\":\"extensionRequest\",\"request_id\":1,\"operation\":{\"kind\":\"run\",\"token\":1,\"id\":2,\"revision\":3,\"query\":\"\",\"prop\":\"onAnything\"}}\n",
        "{\"type\":\"extensionRequest\",\"request_id\":2,\"operation\":{\"kind\":\"start\",\"id\":\"command:x/y\",\"query\":\"\",\"path\":\"/tmp/elsewhere\"}}\n",
    ] {
        assert!(wire::read(&mut Cursor::new(text)).is_err());
    }
}
