// Lifecycle policy ported from Floe PanelLifetime.swift / PanelWork.swift.
// Copyright (Floe) © 2026 René Jiménez. Licensed under GNU AGPLv3.
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
pub mod process;
pub mod wire;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Request {
    Show,
    Toggle,
    Hide,
    BuiltIn(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Packet {
    Ready,
    Request {
        request: Request,
        sequence: u64,
    },
    Idle {
        sequence: u64,
    },
    Shutdown,
    Presented {
        sequence: u64,
        visible: bool,
    },
    SearchVerified,
    PersonalizationRequest {
        request_id: u64,
        operation: asaaan_personalization::Operation,
    },
    PersonalizationReply {
        request_id: u64,
        snapshot: Option<asaaan_personalization::Snapshot>,
        error: Option<String>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Start,
    Terminate,
    Send(Packet),
}
#[derive(Debug, Default, PartialEq)]
enum Phase {
    #[default]
    Stopped,
    Starting,
    TimedOut,
    Ready,
    Closing,
}
#[derive(Debug, Default)]
pub struct Lifetime {
    phase: Phase,
    pending: Vec<Request>,
    sequence: u64,
    failed_starts: u8,
}
impl Lifetime {
    pub fn awaiting_readiness(&self) -> bool {
        self.phase == Phase::Starting
    }
    pub fn request(&mut self, request: Request) -> Vec<Effect> {
        if self.phase == Phase::Stopped && request == Request::Hide {
            return vec![];
        }
        if self.phase == Phase::Ready {
            return vec![self.deliver(request)];
        }
        self.pending.push(request);
        if self.phase != Phase::Stopped {
            return vec![];
        }
        self.phase = Phase::Starting;
        self.failed_starts = 0;
        vec![Effect::Start]
    }
    pub fn ready(&mut self) -> Vec<Effect> {
        if self.phase != Phase::Starting {
            return vec![];
        }
        self.phase = Phase::Ready;
        self.failed_starts = 0;
        let pending = std::mem::take(&mut self.pending);
        pending.into_iter().map(|r| self.deliver(r)).collect()
    }
    pub fn idle(&mut self, sequence: u64) -> Vec<Effect> {
        if self.phase != Phase::Ready || sequence != self.sequence {
            return vec![];
        }
        self.phase = Phase::Closing;
        vec![Effect::Send(Packet::Shutdown)]
    }
    pub fn readiness_timed_out(&mut self) -> Vec<Effect> {
        if self.phase != Phase::Starting {
            return vec![];
        }
        self.phase = Phase::TimedOut;
        vec![Effect::Terminate]
    }
    pub fn exited(&mut self) -> Vec<Effect> {
        if matches!(self.phase, Phase::Starting | Phase::TimedOut) {
            self.failed_starts += 1;
            if self.failed_starts >= 3 {
                self.pending.clear();
            }
        }
        self.phase = if self.pending.is_empty() {
            Phase::Stopped
        } else {
            Phase::Starting
        };
        if self.pending.is_empty() {
            vec![]
        } else {
            vec![Effect::Start]
        }
    }
    fn deliver(&mut self, request: Request) -> Effect {
        self.sequence += 1;
        Effect::Send(Packet::Request {
            request,
            sequence: self.sequence,
        })
    }
}
#[derive(Clone, Default)]
pub struct Work(Arc<AtomicUsize>);
pub struct Lease(Arc<AtomicUsize>);
impl Work {
    pub fn begin(&self) -> Lease {
        self.0.fetch_add(1, Ordering::SeqCst);
        Lease(self.0.clone())
    }
    pub fn is_busy(&self) -> bool {
        self.0.load(Ordering::SeqCst) != 0
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
