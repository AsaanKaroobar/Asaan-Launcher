// Equivalent cases from Floe PanelLifetimeTests.swift and PanelOwnershipTests.swift.
// Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
#![allow(non_snake_case)]
use asaaan_lifecycle::{Effect::*, Lifetime, Packet, Request::*, Work};
fn delivered(request: asaaan_lifecycle::Request, sequence: u64) -> asaaan_lifecycle::Effect {
    Send(Packet::Request { request, sequence })
}
#[test]
fn queuesRequestsUntilReady() {
    let mut p = Lifetime::default();
    assert_eq!(p.request(Show), vec![Start]);
    assert!(p.request(BuiltIn("emoji".into())).is_empty());
    assert_eq!(
        p.ready(),
        vec![delivered(Show, 1), delivered(BuiltIn("emoji".into()), 2)]
    );
    assert!(p.ready().is_empty());
}
#[test]
fn staleIdleCannotCloseAReopenedPanel() {
    let mut p = Lifetime::default();
    p.request(Show);
    p.ready();
    p.request(Toggle);
    assert!(p.idle(1).is_empty());
    assert_eq!(p.idle(2), vec![Send(Packet::Shutdown)]);
}
#[test]
fn openingDuringExitStartsANewProcessOnlyAfterExit() {
    let mut p = Lifetime::default();
    p.request(Show);
    p.ready();
    p.idle(1);
    assert!(p.request(Show).is_empty());
    assert_eq!(p.exited(), vec![Start]);
    assert_eq!(p.ready(), vec![delivered(Show, 2)]);
}
#[test]
fn crashDoesNotReplayAnAlreadyDeliveredCommand() {
    let mut p = Lifetime::default();
    p.request(BuiltIn("sample/run".into()));
    p.ready();
    assert!(p.exited().is_empty());
    assert_eq!(p.request(Show), vec![Start]);
    assert_eq!(p.ready(), vec![delivered(Show, 2)]);
}
#[test]
fn repeatedCrashesBeforeReadinessStopAfterThreeAttempts() {
    let mut p = Lifetime::default();
    assert_eq!(p.request(Show), vec![Start]);
    assert_eq!(p.exited(), vec![Start]);
    assert_eq!(p.exited(), vec![Start]);
    assert!(p.exited().is_empty());
    assert!(p.ready().is_empty());
    assert_eq!(p.request(Toggle), vec![Start]);
    assert_eq!(p.ready(), vec![delivered(Toggle, 1)]);
}
#[test]
fn reachingReadinessResetsTheStartupFailureBudget() {
    let mut p = Lifetime::default();
    p.request(Show);
    p.exited();
    p.exited();
    p.ready();
    p.idle(1);
    p.request(Show);
    assert_eq!(p.exited(), vec![Start]);
    assert_eq!(p.exited(), vec![Start]);
    assert_eq!(p.exited(), vec![Start]);
    assert!(p.exited().is_empty());
}
#[test]
fn aTimeoutKeepsQueuedWorkUntilTheReplacementIsReady() {
    let mut p = Lifetime::default();
    p.request(Show);
    assert_eq!(p.readiness_timed_out(), vec![Terminate]);
    assert!(!p.awaiting_readiness());
    assert!(p.ready().is_empty());
    assert!(p.readiness_timed_out().is_empty());
    assert!(p.request(Toggle).is_empty());
    assert_eq!(p.exited(), vec![Start]);
    assert!(p.awaiting_readiness());
    assert_eq!(p.ready(), vec![delivered(Show, 1), delivered(Toggle, 2)]);
    assert!(p.readiness_timed_out().is_empty());
}
#[test]
fn timeoutsAndCrashesShareABoundedStartupBudget() {
    let mut p = Lifetime::default();
    p.request(Show);
    assert_eq!(p.readiness_timed_out(), vec![Terminate]);
    assert_eq!(p.exited(), vec![Start]);
    assert_eq!(p.exited(), vec![Start]);
    assert_eq!(p.readiness_timed_out(), vec![Terminate]);
    assert!(p.exited().is_empty());
    assert!(p.ready().is_empty());
    assert_eq!(p.request(Toggle), vec![Start]);
    assert_eq!(p.exited(), vec![Start]);
    assert_eq!(p.ready(), vec![delivered(Toggle, 1)]);
}
#[test]
fn threeHangingChildrenDoNotStartAFourth() {
    let mut p = Lifetime::default();
    p.request(Show);
    for attempt in 1..=3 {
        assert_eq!(p.readiness_timed_out(), vec![Terminate]);
        assert_eq!(p.exited(), if attempt < 3 { vec![Start] } else { vec![] });
    }
    assert!(p.ready().is_empty());
}
#[test]
fn hidingAnAbsentPanelDoesNotLaunchIt() {
    let mut p = Lifetime::default();
    assert!(p.request(Hide).is_empty());
    assert!(p.ready().is_empty());
}
#[test]
fn workLeaseEndsWhenItsOwnerFinishes() {
    let w = Work::default();
    let lease = w.begin();
    assert!(w.is_busy());
    drop(lease);
    assert!(!w.is_busy());
}
#[test]
fn asyncWorkHoldsALeaseBeforeTheTaskStarts() {
    let w = Work::default();
    let lease = w.begin();
    assert!(w.is_busy());
    let other = w.clone();
    std::thread::spawn(move || {
        assert!(other.is_busy());
        drop(lease)
    })
    .join()
    .unwrap();
    assert!(!w.is_busy());
}
#[test]
fn concurrentLeasesMustAllFinish() {
    let w = Work::default();
    let a = w.begin();
    let b = w.begin();
    drop(a);
    assert!(w.is_busy());
    drop(b);
    assert!(!w.is_busy());
}
