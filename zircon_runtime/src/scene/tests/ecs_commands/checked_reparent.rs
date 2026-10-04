use super::*;
use crate::core::math::{Transform, Vec3};
use crate::scene::inspection::SubscriptionTable;
use crate::scene::SceneError;
use crate::scene::{NodeKind, SystemStage};
use std::sync::Mutex;
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration};

#[test]
fn deferred_checked_reparent_applies_valid_parent_and_detach() {
    let mut world = World::empty();
    let parent = world.spawn(()).expect("parent fixture must spawn");
    let child = world.spawn(()).expect("child fixture must spawn");

    world.commands().set_parent_checked(child, Some(parent));
    let attach = world.apply_deferred();
    assert!(attach.is_success());
    assert_eq!(attach.applied_count(), 1);
    assert_eq!(world.parent_of(child), Some(parent));

    world.commands().set_parent_checked(child, None);
    let detach = world.apply_deferred();
    assert!(detach.is_success());
    assert_eq!(detach.applied_count(), 1);
    assert_eq!(world.parent_of(child), None);
}

#[test]
fn deferred_checked_reparent_observes_prior_despawn_and_continues_after_error() {
    let mut world = World::empty();
    let parent = world.spawn(()).expect("parent fixture must spawn");
    let child = world.spawn(()).expect("child fixture must spawn");
    world
        .set_parent_checked(child, Some(parent))
        .expect("initial hierarchy must be valid");

    {
        let mut commands = world.commands();
        commands.despawn(parent);
        commands.set_parent_checked(child, Some(parent));
        commands.insert_resource(DeferredBarrierObservation(1));
    }

    let report = world.apply_deferred();
    assert_eq!(report.applied_count(), 3);
    assert_eq!(report.error_count(), 1);
    assert!(!report.is_success());
    assert_eq!(
        report.errors()[0].operation(),
        DeferredCommandOperation::ReparentChecked
    );
    assert_eq!(
        report.errors()[0].target(),
        &DeferredCommandTarget::Resolved(child)
    );
    assert_eq!(
        report.errors()[0].error(),
        &SceneError::MissingParent { child, parent }
    );
    assert!(!world.contains_entity(parent));
    assert!(world.contains_entity(child));
    assert_eq!(world.parent_of(child), None);
    assert_eq!(
        world.get_resource::<DeferredBarrierObservation>(),
        Some(&DeferredBarrierObservation(1))
    );
}

#[test]
fn deferred_checked_reparent_rejects_cycle_without_topology_change() {
    let mut world = World::empty();
    let parent = world
        .spawn_node(NodeKind::Cube)
        .expect("parent fixture must spawn");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("child fixture must spawn");
    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(0.0, 4.0, 0.0)))
        .unwrap();
    world
        .set_parent_checked(child, Some(parent))
        .expect("initial hierarchy must be valid");
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    let subscriptions = Arc::new(Mutex::new(SubscriptionTable::default()));
    subscriptions
        .lock()
        .unwrap()
        .watch(WatchRegistration::new(WatchKey::WorldStructure));
    world.attach_world_sync_subscriptions(Arc::clone(&subscriptions));

    let generation_before = world.world_generation();
    let parent_record_before = world.node_record(parent);
    let child_record_before = world.node_record(child);
    let parent_local_before = world.local_transform(parent);
    let child_local_before = world.local_transform(child);
    let parent_world_before = world.world_transform(parent);
    let child_world_before = world.world_transform(child);
    assert_eq!(
        child_world_before.unwrap().translation,
        Vec3::new(3.0, 4.0, 0.0)
    );

    world.commands().set_parent_checked(parent, Some(child));
    let report = world.apply_deferred();

    assert_eq!(report.applied_count(), 1);
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].operation(),
        DeferredCommandOperation::ReparentChecked
    );
    assert_eq!(
        report.errors()[0].target(),
        &DeferredCommandTarget::Resolved(parent)
    );
    assert_eq!(
        report.errors()[0].error(),
        &SceneError::HierarchyCycle {
            child: parent,
            parent: child,
        }
    );
    assert_eq!(world.parent_of(parent), None);
    assert_eq!(world.parent_of(child), Some(parent));
    assert_eq!(world.node_record(parent), parent_record_before);
    assert_eq!(world.node_record(child), child_record_before);
    assert_eq!(world.local_transform(parent), parent_local_before);
    assert_eq!(world.local_transform(child), child_local_before);
    assert_eq!(world.world_transform(parent), parent_world_before);
    assert_eq!(world.world_transform(child), child_world_before);
    assert!(!world.has_pending_scene_systems());
    assert_eq!(world.world_generation(), generation_before);
    assert_eq!(subscriptions.lock().unwrap().pending_fact_count(), 0);
}

#[test]
fn deferred_checked_reparent_rejects_missing_child_with_resolved_target() {
    let mut world = World::empty();
    let parent = world.spawn(()).expect("parent fixture must spawn");
    let missing_child = u64::MAX;
    let generation_before = world.world_generation();

    world
        .commands()
        .set_parent_checked(missing_child, Some(parent));
    let report = world.apply_deferred();

    assert_eq!(report.applied_count(), 1);
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].operation(),
        DeferredCommandOperation::ReparentChecked
    );
    assert_eq!(
        report.errors()[0].target(),
        &DeferredCommandTarget::Resolved(missing_child)
    );
    assert_eq!(
        report.errors()[0].error(),
        &SceneError::MissingEntity {
            operation: "reparent",
            entity: missing_child,
        }
    );
    assert!(world.contains_entity(parent));
    assert_eq!(world.world_generation(), generation_before);
}
