use super::RuntimeWakeRegistration;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn headless_host_wake_callback_obeys_registration_lifetime() {
    let wakes = Arc::new(AtomicUsize::new(0));
    let target = wakes.clone();
    let mut registration = RuntimeWakeRegistration::register_callback(move || {
        target.fetch_add(1, Ordering::SeqCst);
    });
    let sink = registration.sink();
    unsafe { sink.wake.unwrap()(sink.token) };
    registration.wake();
    registration.unregister();
    unsafe { sink.wake.unwrap()(sink.token) };
    registration.wake();
    assert_eq!(wakes.load(Ordering::SeqCst), 2);
}

#[test]
fn headless_host_wake_callback_panic_is_contained_at_abi_boundary() {
    let registration = RuntimeWakeRegistration::register_callback(|| panic!("test callback panic"));
    let sink = registration.sink();
    unsafe { sink.wake.unwrap()(sink.token) };
}
