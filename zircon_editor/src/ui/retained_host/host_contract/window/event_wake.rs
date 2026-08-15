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
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::HostEventLoopWake;

    #[test]
    fn wake_callback_coalesces_until_the_event_loop_consumes_it() {
        let wake = HostEventLoopWake::default();
        let callback = wake.callback();

        callback();
        callback();

        assert!(wake.take_request());
        assert!(!wake.take_request());
    }

    #[test]
    fn native_wake_is_signaled_only_on_the_pending_edge() {
        let requested = AtomicBool::new(false);

        assert!(super::mark_wake_pending(&requested));
        assert!(!super::mark_wake_pending(&requested));
        assert!(requested.swap(false, std::sync::atomic::Ordering::AcqRel));
        assert!(super::mark_wake_pending(&requested));
    }
}
