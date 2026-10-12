#[test]
fn module_ready_polling_uses_a_bounded_sleep() {
    let source = include_str!("../module_lifecycle.rs");
    let end = source
        .find("mod performance_tests {")
        .expect("performance test module");
    let implementation = &source[..end];

    assert!(implementation.contains("MODULE_READY_POLL_INTERVAL"));
    assert!(implementation.contains("std::thread::sleep"));
    assert!(!implementation.contains("std::thread::yield_now()"));
}
