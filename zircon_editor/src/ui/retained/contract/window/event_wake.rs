use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use winit::event_loop::EventLoopProxy;
use zircon_runtime::core::framework::channel::ChannelWakeCallback;

#[derive(Clone, Default)]
pub(super) struct HostEventLoopWake {
    state: Arc<HostEventLoopWakeState>,
}

#[derive(Default)]
struct HostEventLoopWakeState {
    requested: AtomicBool,
    proxy: Mutex<Option<EventLoopProxy>>,
}

impl HostEventLoopWake {
    pub(super) fn callback(&self) -> ChannelWakeCallback {
        let wake = self.clone();
        Arc::new(move || wake.request())
    }

    pub(super) fn install_proxy(&self, proxy: EventLoopProxy) {
        *self.lock_proxy() = Some(proxy.clone());
        if self.state.requested.load(Ordering::Acquire) {
            proxy.wake_up();
        }
    }

    pub(super) fn clear_proxy(&self) {
        *self.lock_proxy() = None;
    }

    pub(super) fn take_request(&self) -> bool {
        self.state.requested.swap(false, Ordering::AcqRel)
    }

    fn request(&self) {
        if !mark_wake_pending(&self.state.requested) {
            return;
        }
        let proxy = self.lock_proxy().clone();
        if let Some(proxy) = proxy {
            proxy.wake_up();
        }
    }

    fn lock_proxy(&self) -> std::sync::MutexGuard<'_, Option<EventLoopProxy>> {
        self.state
            .proxy
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn mark_wake_pending(requested: &AtomicBool) -> bool {
    !requested.swap(true, Ordering::AcqRel)
}

#[cfg(test)]
#[path = "tests/event_wake.rs"]
mod tests;
