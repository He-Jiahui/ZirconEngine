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
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::HostWindowAttention;

    #[test]
    fn requests_coalesce_until_the_native_event_loop_consumes_them() {
        let wakes = Arc::new(AtomicUsize::new(0));
        let callback_wakes = Arc::clone(&wakes);
        let attention = HostWindowAttention::new(Arc::new(move || {
            callback_wakes.fetch_add(1, Ordering::Relaxed);
        }));

        attention.request();
        attention.request();

        assert!(attention.is_requested());
        assert_eq!(wakes.load(Ordering::Relaxed), 1);
        assert!(attention.take_request());
        assert!(!attention.take_request());
    }
}
