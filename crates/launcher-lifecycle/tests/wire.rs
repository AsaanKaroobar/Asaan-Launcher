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
