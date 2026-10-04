#[test]
fn submission_metrics_sampling_never_flushes_or_waits_for_a_runtime_frame() {
    let sampler = include_str!("../submission_metrics.rs")
        .split("pub fn try_submission_metrics_snapshot")
        .nth(1)
        .and_then(|source| source.split("#[cfg(test)]").next())
        .expect("submission metrics sampler");

    assert!(sampler.contains("self.core.state.try_lock()"));
    assert!(sampler.contains("Err(TryLockError::WouldBlock) => return None"));
    assert!(sampler.contains("state.renderer.submission_metrics()"));
    assert!(!sampler.contains("finish_submission"));
    assert!(!sampler.contains("lock_operation"));
    assert!(!sampler.contains("queue.submit"));
}
