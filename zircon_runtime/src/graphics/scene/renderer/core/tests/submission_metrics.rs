#[test]
fn scene_renderer_forwards_submission_metrics_without_owning_queue_work() {
    let source = include_str!("../submission_metrics.rs");

    assert!(source.contains("self.backend.submission_metrics()"));
    assert!(!source.contains("queue.submit"));
    assert!(!source.contains("queue.write_"));
}
