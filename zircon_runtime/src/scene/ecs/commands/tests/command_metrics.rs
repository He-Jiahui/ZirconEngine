use std::time::Duration;

use super::CommandQueueMetrics;

#[test]
fn ecs_commands_queue_operation_timings_merge_with_transferred_metrics() {
    let mut source = CommandQueueMetrics::default();
    source.record_worker_batch_merge(Duration::from_nanos(7));
    source.record_world_apply(Duration::from_nanos(11));

    let mut destination = CommandQueueMetrics::default();
    destination.merge_from(source);

    assert_eq!(destination.worker_batch_merge_count(), 1);
    assert_eq!(
        destination.worker_batch_merge_duration(),
        Duration::from_nanos(7)
    );
    assert_eq!(destination.world_apply_count(), 1);
    assert_eq!(destination.world_apply_duration(), Duration::from_nanos(11));
}
