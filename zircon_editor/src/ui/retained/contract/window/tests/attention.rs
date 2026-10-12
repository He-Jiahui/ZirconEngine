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
