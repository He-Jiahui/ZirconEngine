#[test]
fn viewport_pick_poll_is_non_blocking_and_pumps_the_backend_timeline() {
    let source = include_str!("../viewport_pick.rs");
    let poll = source
        .split("fn poll_viewport_pick(")
        .nth(1)
        .and_then(|source| source.split("fn cancel_viewport_pick(").next())
        .expect("viewport pick poll function");

    assert!(poll.contains("try_lock()"));
    assert!(poll.contains("TryLockError::WouldBlock"));
    assert!(poll.contains("poll_readback_completions()"));
    assert!(!poll.contains("finish_submission()"));
    assert!(!poll.contains("wait_for_readback_completions()"));
}
