#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BridgeDiagnosticsSnapshot {
    pub enabled_calls: u64,
    pub not_enabled_calls: u64,
}

#[derive(Debug, Default)]
pub(crate) struct BridgeDiagnostics {
    #[cfg(debug_assertions)]
    enabled_calls: AtomicU64,
    #[cfg(debug_assertions)]
    not_enabled_calls: AtomicU64,
}

impl BridgeDiagnostics {
    pub(crate) fn record_enabled_call(&self) {
        #[cfg(debug_assertions)]
        self.enabled_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_not_enabled_call(&self) {
        #[cfg(debug_assertions)]
        self.not_enabled_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn snapshot(&self) -> BridgeDiagnosticsSnapshot {
        BridgeDiagnosticsSnapshot {
            enabled_calls: enabled_calls(self),
            not_enabled_calls: not_enabled_calls(self),
        }
    }
}

#[cfg(debug_assertions)]
fn enabled_calls(diagnostics: &BridgeDiagnostics) -> u64 {
    diagnostics.enabled_calls.load(Ordering::Relaxed)
}

#[cfg(not(debug_assertions))]
fn enabled_calls(_: &BridgeDiagnostics) -> u64 {
    0
}

#[cfg(debug_assertions)]
fn not_enabled_calls(diagnostics: &BridgeDiagnostics) -> u64 {
    diagnostics.not_enabled_calls.load(Ordering::Relaxed)
}

#[cfg(not(debug_assertions))]
fn not_enabled_calls(_: &BridgeDiagnostics) -> u64 {
    0
}
