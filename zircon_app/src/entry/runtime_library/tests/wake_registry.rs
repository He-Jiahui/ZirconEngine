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
    let mut registration = RuntimeWakeRegistration::register(test_proxy(Arc::clone(&wakes), false));
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
    let source = include_str!("../wake_registry.rs");
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
