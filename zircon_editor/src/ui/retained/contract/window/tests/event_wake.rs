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
