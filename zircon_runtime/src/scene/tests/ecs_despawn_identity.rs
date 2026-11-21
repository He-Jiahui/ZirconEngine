use std::cell::Cell;

use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
use crate::scene::components::{Hierarchy, Name, NodeRecord};
use crate::scene::ecs::{
    Component, ComponentLifecycleEvent, InternalEntity, LifecycleEventKind, ObserverId,
};
use crate::scene::{EntityId, NodeKind, SceneError, World};
use zircon_runtime_interface::retained_memory::{
    RetainedBoundBuilderV1, RetainedOwnerV1, RetentionContractError,
};

use super::{SparseMana, StoredHealth};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RestoreReceipt {
    boundary: LifecycleEventKind,
    emitter: EntityId,
    before: InternalEntity,
    after: InternalEntity,
}

thread_local! {
    static SWAP_OBSERVER: Cell<Option<ObserverId>> = const { Cell::new(None) };
    static RESTORE_RECEIPT: Cell<Option<RestoreReceipt>> = const { Cell::new(None) };
    static HEALTH_DESPAWNS: Cell<u32> = const { Cell::new(0) };
    static MANA_REMOVES: Cell<u32> = const { Cell::new(0) };
}

fn reserve_one_restore(
    &(target, _): &(EntityId, EntityId),
    world: &World,
    meter: &mut RetainedBoundBuilderV1,
) -> Result<(), RetentionContractError> {
    // The Empty fixture's only heap payload in its NodeRecord is Name. The
    // observer removes itself before reentry; the two sentinel values are inline.
    meter.array::<NodeRecord>(1)?;
    if let Some(name) = world.get::<Name>(target) {
        name.retained_bound_v1(meter)?;
    }
    Ok(())
}

fn assert_restored_payload(world: &World, target: EntityId, record: &NodeRecord) {
    assert_eq!(world.node_record(target).as_ref(), Some(record));
    assert_eq!(world.get::<StoredHealth>(target), Some(&StoredHealth(71)));
    assert_eq!(world.get::<SparseMana>(target), Some(&SparseMana(72)));
    let mana_id = world.registered_component_id::<SparseMana>().unwrap();
    assert_eq!(world.component_count_for_id(mana_id), 1);
}

fn replace_original_target(
    &(target, emitter): &(EntityId, EntityId),
    world: &mut World,
    event: &ComponentLifecycleEvent,
) {
    assert_eq!(event.entity(), emitter);
    if emitter != target {
        assert_eq!(
            world.get::<Hierarchy>(emitter),
            Some(&Hierarchy { parent: None })
        );
    }
    world
        .remove_observer(SWAP_OBSERVER.take().expect("one-shot swap observer"))
        .unwrap();

    let before = world.internal_entity(target).unwrap();
    let record = world.node_record(target).expect("original target record");
    assert_eq!(record.id, target);
    world
        .remove_entity(target)
        .expect("nested removal must retire the original target");
    assert!(!world.contains_entity(target));
    assert!(!world.contains_internal_entity(before));

    world
        .insert_node_record(record.clone())
        .expect("the retired stable ID can be restored");
    let after = world
        .internal_entity(target)
        .expect("same stable ID restored");
    assert_eq!(after.index(), before.index());
    assert_ne!(after.generation(), before.generation());
    assert_eq!(world.insert(target, StoredHealth(71)), Ok(None));
    assert_eq!(world.insert(target, SparseMana(72)), Ok(None));
    assert_restored_payload(world, target, &record);
    assert_eq!(
        RESTORE_RECEIPT.replace(Some(RestoreReceipt {
            boundary: event.kind(),
            emitter: event.entity(),
            before,
            after,
        })),
        None
    );
}

fn target_fixture() -> (World, EntityId) {
    let mut world = World::empty();
    let target = world.spawn_node(NodeKind::Empty).unwrap();
    world.insert(target, StoredHealth(1)).unwrap();
    world.insert(target, SparseMana(2)).unwrap();
    (world, target)
}

fn assert_despawn_stops_at_replacement<T: Component>(
    world: &mut World,
    target: EntityId,
    emitter: EntityId,
    boundary: LifecycleEventKind,
) {
    let before = world.internal_entity(target).unwrap();
    let record = world.node_record(target).unwrap();
    let mana_id = world.registered_component_id::<SparseMana>().unwrap();
    assert!(world.registered_component_id::<StoredHealth>().unwrap() < mana_id);
    assert_eq!(world.component_count_for_id(mana_id), 1);
    RESTORE_RECEIPT.set(None);
    HEALTH_DESPAWNS.set(0);
    MANA_REMOVES.set(0);

    // Install the loggers first: a Despawn-triggered swap is allowed to finish
    // its current bucket, but the owner must not dispatch the next bucket.
    world.observe_component_lifecycle::<StoredHealth>(
        LifecycleEventKind::Despawn,
        RetainedLifecycleObserverV1::read_only_v1(|_, _| {
            HEALTH_DESPAWNS.set(HEALTH_DESPAWNS.get() + 1);
        }),
    );
    world.observe_component_lifecycle::<SparseMana>(
        LifecycleEventKind::Remove,
        RetainedLifecycleObserverV1::read_only_v1(|_, _| {
            MANA_REMOVES.set(MANA_REMOVES.get() + 1);
        }),
    );
    let handle = world.observe_component_lifecycle::<T>(
        boundary,
        RetainedLifecycleObserverV1::new(
            (target, emitter),
            reserve_one_restore,
            replace_original_target,
        ),
    );
    SWAP_OBSERVER.set(Some(handle));

    let result = world.remove_entity(target);
    let receipt = RESTORE_RECEIPT
        .take()
        .expect("accepted reinsert observed inside callback");
    assert_eq!(
        (receipt.boundary, receipt.emitter, receipt.before),
        (boundary, emitter, before)
    );
    assert_eq!(
        result,
        Err(SceneError::MissingEntity {
            operation: "remove",
            entity: target
        })
    );
    assert!(world.contains_entity(target));
    assert!(!world.contains_internal_entity(before));
    assert_eq!(world.internal_entity(target), Some(receipt.after));
    assert_restored_payload(world, target, &record);

    let expected_health_despawns = if boundary == LifecycleEventKind::Despawn {
        2
    } else {
        1
    };
    assert_eq!(
        HEALTH_DESPAWNS.get(),
        expected_health_despawns,
        "no later outer StoredHealth Despawn bucket"
    );
    assert_eq!(
        MANA_REMOVES.get(),
        1,
        "only the nested removal may reach the later SparseMana bucket"
    );
}

#[test]
fn cpu167_identity_remove_observer_does_not_retarget_same_stable_id() {
    let (mut world, target) = target_fixture();
    assert_despawn_stops_at_replacement::<StoredHealth>(
        &mut world,
        target,
        target,
        LifecycleEventKind::Remove,
    );
}

#[test]
fn cpu167_identity_orphan_replace_observer_does_not_rebind_parent() {
    let (mut world, target) = target_fixture();
    let child = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(child, Some(target)).unwrap();
    assert_eq!(
        world.get::<Hierarchy>(child),
        Some(&Hierarchy {
            parent: Some(target)
        })
    );
    let child_identity = world.internal_entity(child).unwrap();

    assert_despawn_stops_at_replacement::<Hierarchy>(
        &mut world,
        target,
        child,
        LifecycleEventKind::Replace,
    );

    assert_eq!(world.internal_entity(child), Some(child_identity));
    assert_eq!(
        world.get::<Hierarchy>(child),
        Some(&Hierarchy { parent: None })
    );
}

#[test]
fn cpu167_identity_despawn_observer_does_not_continue_to_later_bucket() {
    let (mut world, target) = target_fixture();
    assert_despawn_stops_at_replacement::<StoredHealth>(
        &mut world,
        target,
        target,
        LifecycleEventKind::Despawn,
    );
}
