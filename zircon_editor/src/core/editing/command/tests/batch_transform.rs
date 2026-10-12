use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use zircon_runtime::scene::components::{LocalTransform, NodeKind};
use zircon_runtime::scene::ecs::LifecycleEventKind;
use zircon_runtime_interface::math::{Transform, Vec3};
use zircon_runtime_interface::world_sync::LifecycleTargetChange;

use super::*;

#[test]
fn applied_batch_validation_rejects_world_changes_before_commit() {
    let mut scene = Scene::empty();
    let node_id = scene.spawn_node(NodeKind::Cube).unwrap();
    let before = NodeEditState::capture(&scene, node_id).unwrap();
    let mut after = before.clone();
    after.transform = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
    scene.update_transform(node_id, after.transform).unwrap();
    let expected_world_generation = scene.world_generation();
    let targets = vec![BatchTransformTarget::new(node_id, before, after).unwrap()];

    validate_applied_targets(&scene, &targets, expected_world_generation).unwrap();
    scene.rename_node(node_id, "Externally changed").unwrap();
    assert!(
        validate_applied_targets(&scene, &targets, expected_world_generation).is_err(),
        "an externally advanced world must not commit stale already-applied snapshots"
    );
}

fn cancellation_targets(scene: &mut Scene) -> Vec<BatchTransformTarget> {
    (0..3)
        .map(|index| {
            let node_id = scene.spawn_node(NodeKind::Cube).unwrap();
            let before = NodeEditState::capture(scene, node_id).unwrap();
            let mut after = before.clone();
            after.transform = Transform::from_translation(Vec3::new(20.0 + index as f32, 0.0, 0.0));
            BatchTransformTarget::new(node_id, before, after).unwrap()
        })
        .collect()
}

fn replacement_transform(index: usize) -> Transform {
    Transform::from_translation(Vec3::new(100.0 + index as f32, 0.0, 0.0))
}

fn install_batch_cancellation(
    scene: &mut Scene,
    targets: &[BatchTransformTarget],
    trigger_count: usize,
    cause: LifecycleTargetChange,
    calls: Arc<AtomicUsize>,
) {
    let mut replacement = Scene::empty();
    for (index, target) in targets.iter().enumerate() {
        let node_id = replacement.spawn_node(NodeKind::Cube).unwrap();
        assert_eq!(node_id, target.node_id);
        replacement
            .update_transform(node_id, replacement_transform(index))
            .unwrap();
    }
    let replacement = Mutex::new(Some(replacement));
    let trigger = targets[1].node_id;
    scene.observe_component_lifecycle::<LocalTransform>(
        LifecycleEventKind::Replace,
        move |world, event| {
            if event.entity() != trigger {
                return;
            }
            if calls.fetch_add(1, Ordering::SeqCst) + 1 != trigger_count {
                return;
            }
            match cause {
                LifecycleTargetChange::World => {
                    *world = replacement.lock().unwrap().take().unwrap()
                }
                LifecycleTargetChange::Schedule => *world.schedule_mut() = Default::default(),
            }
        },
    );
}

#[test]
fn primary_lifecycle_cancellation_stops_before_prefix_rollback() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let mut scene = Scene::empty();
        let targets = cancellation_targets(&mut scene);
        let calls = Arc::new(AtomicUsize::new(0));
        install_batch_cancellation(&mut scene, &targets, 1, cause, calls.clone());

        let error = write_targets(&mut scene, &targets, false).unwrap_err();
        let EditCommandError::SceneMutation { source, .. } = error else {
            panic!("primary cancellation must retain its typed SceneError");
        };
        assert_eq!(source.lifecycle_target_change(), Some(cause));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let expected_first = match cause {
            LifecycleTargetChange::World => replacement_transform(0),
            LifecycleTargetChange::Schedule => targets[0].after.transform,
        };
        let expected_last = match cause {
            LifecycleTargetChange::World => replacement_transform(2),
            LifecycleTargetChange::Schedule => targets[2].before.transform,
        };
        assert_eq!(
            scene.find_node(targets[0].node_id).unwrap().transform,
            expected_first
        );
        assert_eq!(
            scene.find_node(targets[2].node_id).unwrap().transform,
            expected_last
        );
    }
}

#[test]
fn secondary_lifecycle_cancellation_stops_remaining_prefix_rollback() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let mut scene = Scene::empty();
        let targets = cancellation_targets(&mut scene);
        scene.remove_entity(targets[2].node_id).unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        install_batch_cancellation(&mut scene, &targets, 2, cause, calls.clone());

        let error = write_targets(&mut scene, &targets, false).unwrap_err();
        let EditCommandError::RollbackFailed {
            command_error,
            rollback_error,
        } = error
        else {
            panic!("a real secondary cancellation must retain both typed failures");
        };
        let EditCommandError::SceneMutation {
            source: primary, ..
        } = *command_error
        else {
            panic!("the failed target must remain typed");
        };
        let EditCommandError::SceneMutation {
            source: secondary, ..
        } = *rollback_error
        else {
            panic!("the rollback cancellation must remain typed");
        };
        assert_eq!(primary.lifecycle_target_change(), None);
        assert_eq!(secondary.lifecycle_target_change(), Some(cause));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let expected_first = match cause {
            LifecycleTargetChange::World => replacement_transform(0),
            LifecycleTargetChange::Schedule => targets[0].after.transform,
        };
        assert_eq!(
            scene.find_node(targets[0].node_id).unwrap().transform,
            expected_first
        );
    }
}

#[test]
fn ordinary_batch_failure_keeps_prefix_recovery() {
    let mut scene = Scene::empty();
    let targets = cancellation_targets(&mut scene);
    scene.remove_entity(targets[2].node_id).unwrap();

    let error = write_targets(&mut scene, &targets, false).unwrap_err();
    assert!(matches!(error, EditCommandError::ExternalEffect { .. }));
    for target in &targets[..2] {
        assert_eq!(
            scene.find_node(target.node_id).unwrap().transform,
            target.before.transform
        );
    }
    assert!(!scene.contains_entity(targets[2].node_id));
}
