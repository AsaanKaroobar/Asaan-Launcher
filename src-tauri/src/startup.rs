//! Opt-in monotonic diagnostics; no paths, queries or preferences are logged.
use std::{sync::OnceLock, time::Instant};
static TRACE: OnceLock<Option<Instant>> = OnceLock::new();
pub fn mark(stage: &str) {
    if let Some(start) = TRACE.get_or_init(|| {
        std::env::var_os("ASAAAN_TRACE_STARTUP")
            .filter(|v| v == "1")
            .map(|_| Instant::now())
    }) {
        eprintln!(
            "ASAAAN_STARTUP pid={} stage={} elapsed_us={}",
            std::process::id(),
            stage,
            start.elapsed().as_micros()
        );
    }
}
