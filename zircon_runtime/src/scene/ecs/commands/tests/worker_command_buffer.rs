use crate::scene::World;

use super::{CommandQueue, WorkerCommandBuffer};

#[test]
fn ecs_commands_successful_worker_barrier_records_one_merge() {
    let mut buffer = WorkerCommandBuffer::with_capacity(0, "commands.metrics", 0);
    buffer.push(|_: &mut World| {});
    let mut queue = CommandQueue::default();

    queue
        .merge_worker_buffers(std::slice::from_mut(&mut buffer))
        .expect("one unique worker key must merge");
    queue.apply(&mut World::empty());

    assert_eq!(queue.metrics().worker_batch_merge_count(), 1);
    assert_eq!(queue.metrics().world_apply_count(), 1);
}

#[test]
fn ecs_commands_empty_worker_barrier_does_not_record_a_merge() {
    let mut queue = CommandQueue::default();

    queue
        .merge_worker_buffers(&mut [])
        .expect("an empty worker slice is a no-op");

    assert_eq!(queue.metrics().worker_batch_merge_count(), 0);
}

#[test]
fn ecs_commands_empty_worker_merge_keeps_prewarms_on_the_producer() {
    let mut worker = WorkerCommandBuffer::with_capacity(0, "commands.empty", 1);
    let mut queue = CommandQueue::default();

    queue
        .merge_worker_buffers(std::slice::from_mut(&mut worker))
        .expect("one empty worker key must merge");

    assert!(queue.is_empty());
    assert!(!queue.has_worker_inline_arenas());
    assert_eq!(queue.metrics().worker_batch_merge_count(), 1);
    assert_eq!(queue.metrics().queue_storage_growths(), 1);
    assert_eq!(queue.metrics().inline_block_storage_growths(), 1);
    assert_eq!(worker.metrics().queue_storage_growths(), 0);
    assert_eq!(worker.metrics().inline_block_storage_growths(), 0);

    let source = include_str!("../worker_command_buffer.rs");
    assert!(source.contains("if !self.arena_is_in_destination {"));
}

#[test]
fn ecs_commands_rejected_worker_barrier_does_not_record_a_merge() {
    let mut buffers = [
        WorkerCommandBuffer::with_capacity(0, "commands.metrics.duplicate", 0),
        WorkerCommandBuffer::with_capacity(0, "commands.metrics.duplicate", 0),
    ];
    let mut queue = CommandQueue::default();

    assert!(queue.merge_worker_buffers(&mut buffers).is_err());

    assert_eq!(queue.metrics().worker_batch_merge_count(), 0);
}
