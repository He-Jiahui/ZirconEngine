use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::scene::World;
use crate::scene::components::Name;
use crate::scene::ecs::{Command, CommandQueue, Component, DeferredCommandOperation, Resource};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[derive(Debug, PartialEq, Eq)]
struct NestedDeferredResource;

impl Resource for NestedDeferredResource {}

#[derive(Debug, PartialEq, Eq)]
struct MergedCommandOrder(Vec<u8>);

impl Resource for MergedCommandOrder {}

struct DropProbe {
    drops: Arc<AtomicUsize>,
}

impl Command for DropProbe {
    fn apply(self, _world: &mut World) {}
}

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

struct SmallNoop(u8);

impl Command for SmallNoop {
    fn apply(self, _world: &mut World) {
        let _ = self.0;
    }
}

struct LargeNoop([u8; 256]);

impl Command for LargeNoop {
    fn apply(self, _world: &mut World) {
        let _ = self.0;
    }
}

#[repr(align(128))]
struct OverAlignedNoop;

impl Command for OverAlignedNoop {
    fn apply(self, _world: &mut World) {}
}

struct EnqueueThenPanic;

impl Command for EnqueueThenPanic {
    fn apply(self, world: &mut World) {
        world.commands().queue_fn(|world| {
            world.insert_resource(NestedDeferredResource);
        });
        panic!("deferred command panic probe");
    }
}

#[test]
fn deferred_command_queue_reuses_its_allocation_after_apply() {
    let source = include_str!("../ecs/commands/command_queue.rs");
    let apply = source
        .split("pub fn apply(&mut self, world: &mut World) -> DeferredCommandReport")
        .nth(1)
        .and_then(|source| source.split("pub fn len(&self)").next())
        .expect("read CommandQueue::apply body");

    assert!(apply.contains("self.commands.drain(..)"));
    assert!(!apply.contains("std::mem::take(&mut self.commands)"));
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
    assert_eq!(metrics.queued_inline_slot_bytes(), 192);
    assert!(metrics.queued_fallback_bytes() >= 256);
    assert_eq!(metrics.fallback_payload_allocations(), 2);
    assert!(metrics.queue_storage_growths() > 0);

    let report = queue.apply(&mut World::empty());

    assert_eq!(report.applied_count(), 3);
    let metrics = queue.metrics();
    assert_eq!(metrics.queued_inline_commands(), 0);
    assert_eq!(metrics.queued_fallback_commands(), 0);
    assert_eq!(metrics.queued_inline_slot_bytes(), 0);
    assert_eq!(metrics.inline_dispatch_calls(), 1);
    assert_eq!(metrics.fallback_dispatch_calls(), 2);
    assert_eq!(metrics.inline_payload_releases(), 1);
    assert_eq!(metrics.fallback_payload_releases(), 2);
}

#[test]
fn deferred_command_queue_budgets_fixed_inline_slot_storage_not_payload_bytes() {
    const INLINE_SLOT_BYTES: usize = 192;
    const INLINE_BUDGET_BYTES: usize = 4 * 1024 * 1024;

    let mut queue = CommandQueue::default();
    for value in 0..(INLINE_BUDGET_BYTES / INLINE_SLOT_BYTES) {
        queue.push(SmallNoop(value as u8));
    }

    queue.push(SmallNoop(0));

    let metrics = queue.metrics();
    assert_eq!(metrics.queued_inline_slot_bytes(), INLINE_BUDGET_BYTES);
    assert_eq!(metrics.queued_fallback_commands(), 1);
    assert_eq!(metrics.queued_fallback_bytes(), 1);
}

#[test]
fn deferred_command_queue_prewarms_reusable_storage_without_per_command_growth() {
    let mut queue = CommandQueue::with_capacity(3);
    assert_eq!(queue.metrics().queue_storage_growths(), 1);

    queue.push(SmallNoop(1));
    queue.push(SmallNoop(2));
    queue.push(SmallNoop(3));

    assert_eq!(queue.metrics().queue_storage_growths(), 1);
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

#[test]
fn deferred_command_success_report_counts_applied_commands_without_errors() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Target".to_string()),)).unwrap();

    {
        let mut commands = world.commands();
        commands.insert(entity, Health(7));
        commands.entity(entity).insert((Marker,));
    }

    let report = world.apply_deferred();

    assert_eq!(report.applied_count(), 2);
    assert_eq!(report.error_count(), 0);
    assert!(report.is_success());
    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
}

#[test]
fn command_queue_on_despawned_entity_target_is_reported_not_silently_dropped() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Removed".to_string()), Health(1)))
        .unwrap();
    assert!(world.remove_entity(entity));

    {
        let mut commands = world.commands();
        commands.insert(entity, Health(2));
        commands.remove::<Health>(entity);
        commands.despawn(entity);
    }

    let report = world.apply_deferred();
    let errors = report.errors();

    assert_eq!(report.applied_count(), 3);
    assert_eq!(report.error_count(), 3);
    assert!(!report.is_success());
    assert_eq!(errors[0].operation(), DeferredCommandOperation::Insert);
    assert_eq!(errors[0].entity(), entity);
    assert!(errors[0].message().contains("missing entity"));
    assert_eq!(errors[1].operation(), DeferredCommandOperation::Remove);
    assert_eq!(errors[1].entity(), entity);
    assert!(errors[1].message().contains("missing entity"));
    assert_eq!(errors[2].operation(), DeferredCommandOperation::Despawn);
    assert_eq!(errors[2].entity(), entity);
    assert!(errors[2].message().contains("missing entity"));
    assert!(!world.has_deferred_commands());
    assert!(world.get::<Health>(entity).is_none());
}
