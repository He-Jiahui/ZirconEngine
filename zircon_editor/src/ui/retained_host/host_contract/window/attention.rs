use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use zircon_runtime::core::framework::channel::ChannelWakeCallback;

/// A coalesced, cross-thread request for the native event loop to take window attention.
///
/// Callers never access the native window directly. They only wake the event loop on the false to
/// true edge, leaving focus execution on the thread which owns the `winit::window::Window`.
#[derive(Clone)]
pub(crate) struct HostWindowAttention {
    requested: Arc<AtomicBool>,
    wake_event_loop: ChannelWakeCallback,
}

impl HostWindowAttention {
    pub(super) fn new(wake_event_loop: ChannelWakeCallback) -> Self {
        Self {
            requested: Arc::new(AtomicBool::new(false)),
            wake_event_loop,
        }
    }

    pub(crate) fn request(&self) {
        if !self.requested.swap(true, Ordering::AcqRel) {
            (self.wake_event_loop)();
        }
    }

    pub(super) fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Acquire)
    }

    pub(super) fn take_request(&self) -> bool {
        self.requested.swap(false, Ordering::AcqRel)
    }
}

#[cfg(test)]
#[path = "tests/attention.rs"]
mod tests;
