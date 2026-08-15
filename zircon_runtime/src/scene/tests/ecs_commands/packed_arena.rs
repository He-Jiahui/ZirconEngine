use super::*;

#[test]
fn deferred_command_queue_reuses_its_allocation_after_apply() {
    let source = include_str!("../../ecs/commands/command_queue.rs");
    let apply = source
        .split("pub fn apply(&mut self, world: &mut World) -> DeferredCommandReport")
        .nth(1)
        .and_then(|source| source.split("pub fn len(&self)").next())
        .expect("read CommandQueue::apply body");

    assert!(apply.contains("commands.clear()"));
    assert!(!apply.contains("std::mem::take(&mut self.commands)"));
}

#[test]
fn ecs_commands_storage_variants_inherit_queue_visibility() {
    let source = include_str!("../../ecs/commands/queued_command.rs");

    assert!(source.contains("pub(super) enum QueuedCommandStorage"));
    assert!(source.contains("payload_bytes: usize"));
    assert!(source.contains("storage_bytes: usize"));
    assert!(source.contains("Fallback(usize)"));
    assert!(!source.contains("pub(super) payload_bytes"));
    assert!(!source.contains("pub(super) storage_bytes"));
    assert!(!source.contains("Fallback(pub(super) usize)"));
}

#[test]
fn ecs_commands_packed_arena_uses_leaf_owners() {
    let wiring = include_str!("../../ecs/commands/mod.rs");
    let arena = include_str!("../../ecs/commands/inline_command_arena.rs");
    let metrics = include_str!("../../ecs/commands/command_metrics.rs");
    let queued = include_str!("../../ecs/commands/queued_command.rs");
    let queue = include_str!("../../ecs/commands/command_queue.rs");

    assert!(wiring.contains("mod command_metrics;"));
    assert!(wiring.contains("mod inline_command_arena;"));
    assert!(wiring.contains("mod queued_command;"));
    assert!(arena.contains("struct InlineCommandBlock"));
    assert!(arena.contains("struct InlineCommandArena"));
    assert!(metrics.contains("pub struct CommandQueueMetrics"));
    assert!(queued.contains("struct InlineCommand"));
    assert!(queued.contains("enum QueuedCommand"));
    assert!(queue.lines().count() <= 800);
    assert!(!queue.contains("struct InlineCommandBlock"));
    assert!(!queue.contains("struct InlineCommandArena"));
    assert!(!queue.contains("struct CommandQueueMetrics"));
    assert!(!queue.contains("impl CommandQueueMetrics"));
    assert!(!queue.contains("enum QueuedCommand"));
}

#[test]
fn ecs_commands_structural_barrier_uses_typed_api_transaction_owner() {
    let source = include_str!("../../world/deferred_structural_segment.rs");

    assert!(source.contains(
        "use super::typed_api::{BundleInsertionTransaction, DeferredBundleTransactionArtifact};"
    ));
    assert!(!source.contains("use super::{BundleInsertionTransaction"));
    assert!(source.contains("fn begin_transaction<'world>("));
    assert!(source.contains("world: &'world mut World"));
    assert!(source.contains("SceneResult<BundleInsertionTransaction<'world>>"));
}

#[test]
fn deferred_command_queue_inlines_small_commands_and_reports_explicit_fallbacks() {
    let mut queue = CommandQueue::default();
    queue.push(SmallNoop(1));
    queue.push(LargeNoop([0; 256]));
    queue.push(OverAlignedNoop);

    let metrics = queue.metrics();
    assert_eq!(metrics.queued_inline_commands(), 1);
    assert_eq!(metrics.queued_fallback_commands(), 2);
    assert!(metrics.queued_inline_bytes() > 0);
    assert_eq!(metrics.queued_inline_storage_bytes(), 1);
    assert!(metrics.queued_fallback_bytes() >= 256);
    assert_eq!(metrics.fallback_payload_allocations(), 1);
    assert!(metrics.queue_storage_growths() > 0);

    let report = queue.apply(&mut World::empty());

    assert_eq!(report.applied_count(), 3);
    let metrics = queue.metrics();
    assert_eq!(metrics.queued_inline_commands(), 0);
    assert_eq!(metrics.queued_fallback_commands(), 0);
    assert_eq!(metrics.queued_inline_storage_bytes(), 0);
    assert_eq!(metrics.inline_dispatch_calls(), 1);
    assert_eq!(metrics.fallback_dispatch_calls(), 2);
    assert_eq!(metrics.inline_payload_releases(), 1);
    assert_eq!(metrics.fallback_payload_releases(), 1);
}

#[test]
fn deferred_command_queue_packs_one_hundred_thousand_small_commands_without_fallbacks() {
    let mut queue = CommandQueue::with_capacity(100_000);
    for value in 0..100_000 {
        queue.push(SmallNoop(value as u8));
    }

    let metrics = queue.metrics();
    assert_eq!(metrics.queued_inline_commands(), 100_000);
    assert_eq!(metrics.queued_inline_bytes(), 100_000);
    assert_eq!(metrics.queued_inline_storage_bytes(), 100_000);
    assert_eq!(metrics.queued_fallback_commands(), 0);
    assert_eq!(metrics.fallback_payload_allocations(), 0);
    assert!(metrics.inline_block_storage_growths() <= 2);
}

#[test]
fn deferred_command_queue_packs_and_dispatches_cache_aligned_payloads() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut queue = CommandQueue::default();
    queue.push(SmallNoop(1));
    queue.push(CacheAlignedNoop {
        calls: Arc::clone(&calls),
    });

    let metrics = queue.metrics();
    assert_eq!(metrics.queued_inline_commands(), 2);
    assert_eq!(metrics.queued_inline_bytes(), 65);
    assert_eq!(metrics.queued_inline_storage_bytes(), 128);
    assert_eq!(metrics.queued_fallback_commands(), 0);

    let report = queue.apply(&mut World::empty());

    assert_eq!(report.applied_count(), 2);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn deferred_command_queue_bounds_packed_inline_arena_and_falls_back_explicitly() {
    const INLINE_BLOCK_BYTES: usize = 64 * 1024;
    const INLINE_BUDGET_BYTES: usize = 4 * 1024 * 1024;
    const COMMANDS_PER_BLOCK: usize = INLINE_BLOCK_BYTES / 192;
    const INLINE_BLOCK_COUNT: usize = INLINE_BUDGET_BYTES / INLINE_BLOCK_BYTES;

    let mut queue = CommandQueue::default();
    for _ in 0..(COMMANDS_PER_BLOCK * INLINE_BLOCK_COUNT) {
        queue.push(MaxInlineNoop([0; 192]));
    }
    queue.push(MaxInlineNoop([0; 192]));

    let metrics = queue.metrics();
    assert_eq!(metrics.queued_inline_commands(), 21_824);
    assert_eq!(metrics.queued_inline_storage_bytes(), 4_194_240);
    assert_eq!(metrics.queued_fallback_commands(), 1);
    assert_eq!(metrics.queued_fallback_bytes(), 192);
}

#[test]
fn deferred_command_queue_prewarms_reusable_storage_without_per_command_growth() {
    let mut queue = CommandQueue::with_capacity(3);
    assert_eq!(queue.metrics().queue_storage_growths(), 1);
    assert_eq!(queue.metrics().inline_block_storage_growths(), 1);

    queue.push(SmallNoop(1));
    queue.push(SmallNoop(2));
    queue.push(SmallNoop(3));

    assert_eq!(queue.metrics().queue_storage_growths(), 1);
    assert_eq!(queue.metrics().inline_block_storage_growths(), 1);
}

#[test]
fn deferred_command_queue_keeps_large_arena_capacity_after_a_small_merge() {
    const COMMANDS_PER_BLOCK: usize = (64 * 1024) / 192;

    let mut destination = CommandQueue::default();
    for _ in 0..=COMMANDS_PER_BLOCK {
        destination.push(MaxInlineNoop([0; 192]));
    }
    destination.apply(&mut World::empty());

    let mut nested = CommandQueue::default();
    nested.push(SmallNoop(1));
    destination.append(&mut nested);
    destination.apply(&mut World::empty());
    let growths_before_reuse = destination.metrics().inline_block_storage_growths();

    for _ in 0..=COMMANDS_PER_BLOCK {
        destination.push(MaxInlineNoop([0; 192]));
    }

    assert_eq!(
        destination.metrics().inline_block_storage_growths(),
        growths_before_reuse
    );
}

#[test]
fn deferred_command_queue_merge_keeps_worker_buffer_order_deterministic() {
    let mut first = CommandQueue::default();
    let mut second = CommandQueue::default();
    first.push(|world: &mut World| {
        world.insert_resource(MergedCommandOrder(vec![1]));
    });
    second.push(|world: &mut World| {
        world
            .get_resource_mut::<MergedCommandOrder>()
            .expect("first local buffer should run first")
            .0
            .push(2);
    });

    first.append(&mut second);
    let mut world = World::empty();
    let report = first.apply(&mut world);

    assert_eq!(report.applied_count(), 2);
    assert!(second.is_empty());
    assert_eq!(
        world.get_resource::<MergedCommandOrder>(),
        Some(&MergedCommandOrder(vec![1, 2]))
    );
}

#[test]
fn deferred_command_queue_merge_records_destination_storage_growth() {
    let mut destination = CommandQueue::default();
    let mut worker_queue = CommandQueue::with_capacity(2);
    worker_queue.push(SmallNoop(1));
    worker_queue.push(SmallNoop(2));

    destination.append(&mut worker_queue);

    assert_eq!(destination.metrics().queue_storage_growths(), 2);
    assert!(worker_queue.is_empty());
}

#[test]
fn deferred_command_queue_merges_sixty_four_worker_arenas_without_per_item_fallbacks() {
    const WORKER_COUNT: usize = 64;
    const COMMANDS_PER_WORKER: usize = 100;

    let mut workers = (0..WORKER_COUNT)
        .map(|worker| {
            let mut queue = CommandQueue::with_capacity(COMMANDS_PER_WORKER);
            for command in 0..COMMANDS_PER_WORKER {
                queue.push(SmallNoop((worker + command) as u8));
            }
            queue
        })
        .collect::<Vec<_>>();
    let mut merged = CommandQueue::with_capacity(WORKER_COUNT * COMMANDS_PER_WORKER);

    for worker in &mut workers {
        merged.append(worker);
    }

    let metrics = merged.metrics();
    assert_eq!(metrics.queued_inline_commands(), 6_400);
    assert_eq!(metrics.queued_fallback_commands(), 0);
    assert_eq!(metrics.fallback_payload_allocations(), 0);
    assert!(metrics.queued_inline_storage_bytes() <= 4 * 1024 * 1024);
    assert!(workers.iter().all(CommandQueue::is_empty));

    let report = merged.apply(&mut World::empty());

    assert_eq!(report.applied_count(), 6_400);
}

#[test]
fn ecs_commands_worker_barrier_metrics_record_one_merge_and_apply_at_each_lane_width() {
    for worker_count in [1_usize, 8, 64] {
        let mut world = World::empty();
        let mut buffers = (0..worker_count)
            .map(|worker| {
                let mut buffer = WorkerCommandBuffer::with_capacity(
                    worker as i32,
                    format!("tests.metrics.worker.{worker}"),
                    1,
                );
                buffer.push(SmallNoop(worker as u8));
                buffer
            })
            .collect::<Vec<_>>();
        let mut buffer_refs = buffers.iter_mut().collect::<Vec<_>>();

        world
            .merge_worker_command_buffers(&mut buffer_refs)
            .expect("distinct worker keys must merge at one barrier");
        assert_eq!(
            world.deferred_command_metrics().worker_batch_merge_count(),
            1
        );
        assert_eq!(world.deferred_command_metrics().world_apply_count(), 0);

        let report = world.apply_deferred();

        assert_eq!(report.applied_count(), worker_count);
        assert_eq!(
            world.deferred_command_metrics().worker_batch_merge_count(),
            1
        );
        assert_eq!(world.deferred_command_metrics().world_apply_count(), 1);

        let empty_report = world.apply_deferred();

        assert_eq!(empty_report.applied_count(), 0);
        assert_eq!(
            world.deferred_command_metrics().worker_batch_merge_count(),
            1
        );
        assert_eq!(world.deferred_command_metrics().world_apply_count(), 1);
    }
}

#[test]
fn deferred_command_queue_releases_unapplied_inline_payloads() {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut queue = CommandQueue::default();
    queue.push(DropProbe {
        drops: Arc::clone(&drops),
    });

    drop(queue);

    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn deferred_command_panic_releases_payload_and_preserves_nested_enqueue_for_next_apply() {
    let mut world = World::empty();
    let drops = Arc::new(AtomicUsize::new(0));
    world.commands().queue(EnqueueThenPanic);
    world.commands().queue(DropProbe {
        drops: Arc::clone(&drops),
    });

    let panic = catch_unwind(AssertUnwindSafe(|| world.apply_deferred()));

    assert!(panic.is_err());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(world.has_deferred_commands());
    assert!(world.get_resource::<NestedDeferredResource>().is_none());

    let report = world.apply_deferred();

    assert!(report.is_success());
    assert_eq!(report.applied_count(), 1);
    assert_eq!(
        world.get_resource::<NestedDeferredResource>(),
        Some(&NestedDeferredResource)
    );
}
