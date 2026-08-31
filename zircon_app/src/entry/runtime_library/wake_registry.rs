use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use winit::event_loop::EventLoopProxy;
use zircon_runtime_interface::ZrRuntimeWakeSinkV1;

static NEXT_WAKE_TOKEN: AtomicU64 = AtomicU64::new(1);
static WAKE_REGISTRY: OnceLock<Mutex<HashMap<u64, EventLoopProxy>>> = OnceLock::new();

pub(in crate::entry) struct RuntimeWakeRegistration {
    token: u64,
    proxy: EventLoopProxy,
}

impl RuntimeWakeRegistration {
    pub(in crate::entry) fn register(proxy: EventLoopProxy) -> Self {
        loop {
            let token = NEXT_WAKE_TOKEN.fetch_add(1, Ordering::Relaxed);
            if token == 0 {
                continue;
            }
            let mut registry = lock_registry();
            if let Entry::Vacant(entry) = registry.entry(token) {
                entry.insert(proxy.clone());
                return Self { token, proxy };
            }
        }
    }

    pub(super) fn sink(&self) -> ZrRuntimeWakeSinkV1 {
        ZrRuntimeWakeSinkV1::new(self.token, runtime_wake_trampoline)
    }

    pub(super) fn wake(&self) {
        if self.token != 0 {
            self.proxy.wake_up();
        }
    }

    pub(super) fn unregister(&mut self) {
        if self.token == 0 {
            return;
        }
        lock_registry().remove(&self.token);
        self.token = 0;
    }
}

impl Drop for RuntimeWakeRegistration {
    fn drop(&mut self) {
        self.unregister();
    }
}

unsafe extern "C" fn runtime_wake_trampoline(token: u64) {
    let _ = catch_unwind(AssertUnwindSafe(|| wake_token(token)));
}

fn wake_token(token: u64) {
    let proxy = lock_registry().get(&token).cloned();
    if let Some(proxy) = proxy {
        proxy.wake_up();
    }
}

fn lock_registry() -> MutexGuard<'static, HashMap<u64, EventLoopProxy>> {
    WAKE_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use winit::event_loop::{EventLoopProxy, EventLoopProxyProvider};

    use super::{runtime_wake_trampoline, RuntimeWakeRegistration};

    struct CountingWakeTarget {
        wakes: Arc<AtomicUsize>,
        panic_on_wake: bool,
    }

    impl fmt::Debug for CountingWakeTarget {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.debug_struct("CountingWakeTarget").finish()
        }
    }

    impl EventLoopProxyProvider for CountingWakeTarget {
        fn wake_up(&self) {
            self.wakes.fetch_add(1, Ordering::SeqCst);
            assert!(!self.panic_on_wake, "test wake target panic");
        }
    }

    fn test_proxy(wakes: Arc<AtomicUsize>, panic_on_wake: bool) -> EventLoopProxy {
        EventLoopProxy::new(Arc::new(CountingWakeTarget {
            wakes,
            panic_on_wake,
        }))
    }

    #[test]
    fn runtime_wake_registration_routes_only_while_token_is_registered() {
        let wakes = Arc::new(AtomicUsize::new(0));
        let mut registration =
            RuntimeWakeRegistration::register(test_proxy(Arc::clone(&wakes), false));
        let sink = registration.sink();
        assert!(sink.is_valid());

        registration.wake();
        unsafe { sink.wake.unwrap()(sink.token) };
        assert_eq!(wakes.load(Ordering::SeqCst), 2);

        registration.unregister();
        registration.wake();
        unsafe { sink.wake.unwrap()(sink.token) };
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn host_wake_uses_the_registration_owned_proxy() {
        let source = include_str!("wake_registry.rs");
        let wake_body = source
            .split("pub(super) fn wake(&self) {")
            .nth(1)
            .and_then(|tail| tail.split("\n    }").next())
            .expect("wake method source");

        assert!(wake_body.contains("self.proxy.wake_up()"));
        assert!(!wake_body.contains("wake_token"));
    }

    #[test]
    fn runtime_wake_trampoline_contains_host_proxy_panics() {
        let wakes = Arc::new(AtomicUsize::new(0));
        let registration = RuntimeWakeRegistration::register(test_proxy(Arc::clone(&wakes), true));
        let sink = registration.sink();

        let result = std::panic::catch_unwind(|| unsafe {
            runtime_wake_trampoline(sink.token);
        });

        assert!(result.is_ok());
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
    }
}
