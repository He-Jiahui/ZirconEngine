use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
use crate::scene::ecs::{Component, ComponentLifecycleEvent, LifecycleEventKind, StorageType};
use crate::scene::{EntityId, World};

use super::Health;

#[derive(Debug, PartialEq, Eq)]
struct LateSparse(u32);

zircon_runtime_interface::impl_retained_owner_v1!(LateSparse { 0 });
impl Component for LateSparse {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

// The callback is synchronous. Keep its receipt outside World so skipping it cannot look
// like successful cleanup, without retaining state in the inline-payload observers.
thread_local! {
    static SPARSE_WRITE_EVENT: Cell<Option<(LifecycleEventKind, EntityId)>> = const { Cell::new(None) };
}

fn record_sparse_write(event: &ComponentLifecycleEvent) {
    assert_eq!(
        SPARSE_WRITE_EVENT.replace(Some((event.kind(), event.entity()))),
        None
    );
}

fn despawn_with_confirmed_sparse_write(
    world: &mut World,
    target: EntityId,
    kind: LifecycleEventKind,
    callback: fn(&(), &mut World, &ComponentLifecycleEvent),
) {
    SPARSE_WRITE_EVENT.set(None);
    world.observe_component_lifecycle::<Health>(
        kind,
        RetainedLifecycleObserverV1::new((), |_, _, _| Ok(()), callback),
    );
    world.remove_entity(target).unwrap();
    assert_eq!(SPARSE_WRITE_EVENT.take(), Some((kind, target)));
    assert!(!world.contains_entity(target));
}

fn assert_late_sparse_insert_is_cleaned_up(kind: LifecycleEventKind) {
    let mut world = World::empty();
    let late_id = world.component_id::<LateSparse>();
    let target = world.spawn((Health(1),)).unwrap();
    assert_eq!(world.get::<LateSparse>(target), None);
    assert_eq!(world.component_count_for_id(late_id), 0);

    despawn_with_confirmed_sparse_write(&mut world, target, kind, |_, world, event| {
        let entity = event.entity();
        assert_eq!(world.insert(entity, LateSparse(7)), Ok(None));
        assert_eq!(world.get::<LateSparse>(entity), Some(&LateSparse(7)));
        record_sparse_write(event);
    });

    assert_eq!(world.get::<LateSparse>(target), None);
    assert_eq!(
        world.component_count_for_id(late_id),
        0,
        "{kind:?} observer's accepted insertion must not leave an owned sparse row"
    );
}

#[test]
fn cpu167_despawn_cleans_up_sparse_added_by_remove_observer() {
    assert_late_sparse_insert_is_cleaned_up(LifecycleEventKind::Remove);
}

#[test]
fn cpu167_despawn_cleans_up_sparse_added_by_despawn_observer() {
    assert_late_sparse_insert_is_cleaned_up(LifecycleEventKind::Despawn);
}

#[test]
fn cpu167_despawn_preserves_survivor_with_preexisting_sparse() {
    let mut world = World::empty();
    let late_id = world.component_id::<LateSparse>();
    let target = world.spawn((Health(1), LateSparse(3))).unwrap();
    let survivor = world.spawn((LateSparse(9),)).unwrap();
    assert_eq!(world.component_count_for_id(late_id), 2);

    world.remove_entity(target).unwrap();

    assert!(!world.contains_entity(target));
    assert_eq!(world.get::<LateSparse>(target), None);
    assert!(world.contains_entity(survivor));
    assert_eq!(world.get::<LateSparse>(survivor), Some(&LateSparse(9)));
    assert_eq!(world.component_count_for_id(late_id), 1);
    world.remove_entity(survivor).unwrap();
    assert_eq!(world.component_count_for_id(late_id), 0);
}

#[test]
fn cpu167_despawn_cleans_up_sparse_replaced_by_remove_observer() {
    let mut world = World::empty();
    let late_id = world.component_id::<LateSparse>();
    let target = world.spawn((Health(1), LateSparse(3))).unwrap();
    let survivor = world.spawn((LateSparse(9),)).unwrap();
    assert_eq!(world.component_count_for_id(late_id), 2);

    despawn_with_confirmed_sparse_write(
        &mut world,
        target,
        LifecycleEventKind::Remove,
        |_, world, event| {
            let entity = event.entity();
            assert_eq!(world.insert(entity, LateSparse(7)), Ok(Some(LateSparse(3))));
            assert_eq!(world.get::<LateSparse>(entity), Some(&LateSparse(7)));
            record_sparse_write(event);
        },
    );

    assert_eq!(world.get::<LateSparse>(target), None);
    assert!(world.contains_entity(survivor));
    assert_eq!(world.get::<LateSparse>(survivor), Some(&LateSparse(9)));
    assert_eq!(world.component_count_for_id(late_id), 1);
}

struct DropTrackedLateSparse(u32, Arc<AtomicUsize>);

zircon_runtime_interface::impl_retained_owner_v1!(DropTrackedLateSparse { 0, 1 });
impl Component for DropTrackedLateSparse {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

impl Drop for DropTrackedLateSparse {
    fn drop(&mut self) {
        self.1.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn cpu167_despawn_drops_callback_added_sparse_once() {
    // Pass the test's counter into the metered payload, not retained observer state.
    thread_local! {
        static TARGET_DROPS: Cell<Option<Arc<AtomicUsize>>> = const { Cell::new(None) };
    }

    let mut world = World::empty();
    let late_id = world.component_id::<DropTrackedLateSparse>();
    let target_drops = Arc::new(AtomicUsize::new(0));
    let survivor_drops = Arc::new(AtomicUsize::new(0));
    let target = world.spawn((Health(1),)).unwrap();
    let survivor = world
        .spawn((DropTrackedLateSparse(9, Arc::clone(&survivor_drops)),))
        .unwrap();
    assert!(world.get::<DropTrackedLateSparse>(target).is_none());
    assert_eq!(world.component_count_for_id(late_id), 1);
    TARGET_DROPS.set(Some(Arc::clone(&target_drops)));

    despawn_with_confirmed_sparse_write(
        &mut world,
        target,
        LifecycleEventKind::Remove,
        |_, world, event| {
            let entity = event.entity();
            let drops = TARGET_DROPS.take().expect("test drop counter");
            assert!(world
                .insert(entity, DropTrackedLateSparse(7, drops))
                .expect("Remove observer must insert its sparse payload")
                .is_none());
            assert_eq!(
                world.get::<DropTrackedLateSparse>(entity).map(|row| row.0),
                Some(7)
            );
            record_sparse_write(event);
        },
    );

    assert!(world.get::<DropTrackedLateSparse>(target).is_none());
    assert_eq!(target_drops.load(Ordering::Relaxed), 1);
    assert_eq!(survivor_drops.load(Ordering::Relaxed), 0);
    assert!(world.contains_entity(survivor));
    assert_eq!(
        world
            .get::<DropTrackedLateSparse>(survivor)
            .map(|row| row.0),
        Some(9)
    );
    assert_eq!(world.component_count_for_id(late_id), 1);
    drop(world);
    assert_eq!(target_drops.load(Ordering::Relaxed), 1);
    assert_eq!(survivor_drops.load(Ordering::Relaxed), 1);
}
