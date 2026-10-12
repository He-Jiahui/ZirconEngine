use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use zircon_runtime::scene::components::{Name, NodeKind};
use zircon_runtime::scene::ecs::{Component, LifecycleEventKind};
use zircon_runtime::scene::{DefaultLevelManager, LevelSystem, NodeId, SceneError};
use zircon_runtime_interface::world_sync::LifecycleTargetChange;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    CommandEffect, EditCommand, EditCommandError, EditorTransactionEngine, HistoryContextId,
};
use crate::core::editing::intent::EditorIntent;
use crate::core::editing::selection::SceneSelection;
use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::tests::editing::support::{cube_and_camera, cube_id, test_state};
use crate::ui::workbench::state::EditorStateOperationError;

#[test]
fn delete_node_is_undoable() {
    let mut state = test_state();
    let cube = cube_id(&state);

    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());
    assert!(state
        .world
        .expect_with_world(|scene| scene.find_node(cube).is_none()));

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert!(state
        .world
        .expect_with_world(|scene| scene.find_node(cube).is_some()));
}

#[test]
fn delete_undo_keeps_the_inverse_delta_after_restore_preflight_rejection() {
    let mut state = test_state();
    let cube = cube_id(&state);

    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());
    let replacement = DefaultLevelManager::default().create_default_level();
    let conflict_batch = replacement.with_world_mut(|fixture| {
        assert!(fixture.contains_entity(cube));
        fixture
            .remove_entity_recursive(cube)
            .expect("detach the same-ID fixture node")
    });
    state.world.expect_with_world_mut(|scene| {
        scene
            .restore_detached_entity_batch(conflict_batch)
            .expect("occupy the detached entity id before undo");
    });

    let error = state
        .apply_intent(EditorIntent::Undo)
        .expect_err("the restore preflight must reject the conflicting entity");
    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::SceneMutation {
            operation: "restore detached entity batch",
            source: SceneError::DuplicateEntity { entity },
        }) if entity == cube
    ));

    state.world.expect_with_world_mut(|scene| {
        scene
            .remove_entity(cube)
            .expect("remove the conflicting replacement");
    });
    assert!(
        state.apply_intent(EditorIntent::Undo).unwrap(),
        "the rejected restore must retain the exact move-only inverse delta for retry"
    );
    assert!(state
        .world
        .expect_with_world(|scene| scene.find_node(cube).is_some()));
}

#[test]
fn deleting_last_camera_is_rejected() {
    let mut state = test_state();
    let camera = state.world.expect_with_world(|scene| scene.active_camera());

    let error = state
        .apply_intent(EditorIntent::DeleteNode(camera))
        .unwrap_err();

    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::InvariantViolation {
            invariant: "cannot delete the last remaining camera"
        })
    ));
    assert!(state
        .world
        .expect_with_world(|scene| scene.find_node(camera).is_some()));
}

#[test]
fn deleting_parent_of_the_last_camera_is_rejected() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::SetParent(camera, Some(cube)))
        .unwrap());

    let error = state
        .apply_intent(EditorIntent::DeleteNode(cube))
        .expect_err("deleting a parent must not remove the last camera descendant");

    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::InvariantViolation {
            invariant: "cannot delete the last remaining camera"
        })
    ));
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(camera).is_some());
    });
}

#[derive(Debug)]
struct DeletePayloadDropProbe(Arc<AtomicUsize>);

impl Component for DeletePayloadDropProbe {}

impl Drop for DeletePayloadDropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

fn delete_payload_context(drops: Arc<AtomicUsize>) -> (LevelSystem, CoreEditContext, NodeId) {
    let level = DefaultLevelManager::default().create_default_level();
    let cube = level.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id
    });
    level.with_world_mut(|scene| scene.insert(cube, DeletePayloadDropProbe(drops)).unwrap());
    let mut context = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());
    context
        .bind_scene(level.clone(), SceneSelection::new(vec![cube], Some(cube)))
        .unwrap();
    (level, context, cube)
}

fn install_delete_target_change(
    level: &LevelSystem,
    cube: NodeId,
    kind: LifecycleEventKind,
    cause: LifecycleTargetChange,
    calls: Arc<AtomicUsize>,
) {
    let replacement = DefaultLevelManager::default()
        .create_default_level()
        .with_world_mut(std::mem::take);
    let replacement = Mutex::new(Some(replacement));
    level.with_world_mut(|scene| {
        scene.observe_component_lifecycle::<Name>(kind, move |world, event| {
            if event.entity() != cube {
                return;
            }
            calls.fetch_add(1, Ordering::SeqCst);
            match cause {
                LifecycleTargetChange::World => {
                    *world = replacement.lock().unwrap().take().unwrap()
                }
                LifecycleTargetChange::Schedule => *world.schedule_mut() = Default::default(),
            }
        });
    });
}

fn assert_delete_interrupted(error: &EditCommandError, cause: LifecycleTargetChange) {
    let EditCommandError::LifecycleInterrupted {
        source,
        post_callback_errors,
    } = error
    else {
        panic!("delete cancellation must remain persistent without invented rollback");
    };
    assert!(post_callback_errors.is_empty());
    let EditCommandError::SceneMutation { source, .. } = source.as_ref() else {
        panic!("the runtime cancellation must retain its typed source");
    };
    assert_eq!(source.lifecycle_target_change(), Some(cause));
}

#[test]
fn cancelled_delete_keeps_move_only_payload_in_the_faulted_engine() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let drops = Arc::new(AtomicUsize::new(0));
        let (level, context, cube) = delete_payload_context(drops.clone());
        let original_name = level.with_world(|scene| scene.find_node(cube).unwrap().name.clone());
        let rename = level
            .with_world(|scene| {
                EditorCommand::rename_node(scene, cube, "Applied prefix name".to_owned())
            })
            .unwrap()
            .unwrap();
        let engine = EditorTransactionEngine::new(context);
        let mut scope = engine
            .begin("delete with retained payload", HistoryContextId::Global)
            .unwrap();
        scope.push(rename).unwrap();
        let command = level
            .with_world(|scene| EditorCommand::delete_node(scene, cube))
            .unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        install_delete_target_change(
            &level,
            cube,
            LifecycleEventKind::Remove,
            cause,
            calls.clone(),
        );

        let error = scope.push(command).unwrap_err();
        assert_delete_interrupted(&error, cause);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            drops.load(Ordering::SeqCst),
            0,
            "the failed local command must be retained"
        );
        drop(scope);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "scope Drop must stop at the fault gate"
        );
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(matches!(
            engine.undo(HistoryContextId::Global),
            Err(EditCommandError::EngineFaulted { .. })
        ));
        level.with_world(|scene| match cause {
            LifecycleTargetChange::World => {
                assert_eq!(scene.find_node(cube).unwrap().name, original_name)
            }
            LifecycleTargetChange::Schedule => assert!(!scene.contains_entity(cube)),
        });
        drop(engine);
        assert_eq!(
            drops.load(Ordering::SeqCst),
            1,
            "explicit engine disposal drops the exact payload once"
        );
    }
}

#[test]
fn cancelled_restore_does_not_fabricate_a_retry_batch() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let drops = Arc::new(AtomicUsize::new(0));
        let (level, mut context, cube) = delete_payload_context(drops.clone());
        let mut command = level
            .with_world(|scene| EditorCommand::delete_node(scene, cube))
            .unwrap();
        command.apply(&mut context).unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        let before = context.selection_snapshot();
        let calls = Arc::new(AtomicUsize::new(0));
        install_delete_target_change(&level, cube, LifecycleEventKind::Add, cause, calls.clone());

        let error = command.revert(&mut context).unwrap_err();
        assert_eq!(error.effect, CommandEffect::Interrupted);
        assert_delete_interrupted(&error.source, cause);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(context.selection_snapshot(), before);
        let expected_drops = match cause {
            LifecycleTargetChange::World => 1,
            LifecycleTargetChange::Schedule => 0,
        };
        assert_eq!(drops.load(Ordering::SeqCst), expected_drops);
        // This only exercises the command's local no-batch admission guard.
        let rejected_retry = command.revert(&mut context).unwrap_err();
        assert_eq!(rejected_retry.effect, CommandEffect::Unchanged);
        assert!(matches!(
            rejected_retry.source,
            EditCommandError::InvariantViolation {
                invariant: "delete command must retain a detached batch before it can be reverted",
            }
        ));
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "no callback or old-world retry is allowed"
        );
        drop(command);
        assert_eq!(drops.load(Ordering::SeqCst), expected_drops);
        drop(context);
        drop(level);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn restore_admission_failure_keeps_the_exact_move_only_batch_for_retry() {
    let drops = Arc::new(AtomicUsize::new(0));
    let (level, mut context, cube) = delete_payload_context(drops.clone());
    let mut command = level
        .with_world(|scene| EditorCommand::delete_node(scene, cube))
        .unwrap();
    command.apply(&mut context).unwrap();
    let mut unavailable = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());

    let error = command.revert(&mut unavailable).unwrap_err();
    assert_eq!(error.effect, CommandEffect::Unchanged);
    assert!(!error.source.is_interrupted());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert!(level.with_world(|scene| !scene.contains_entity(cube)));
    command
        .revert(&mut context)
        .expect("a callback that was never entered must keep its exact batch");
    assert!(level.with_world(|scene| scene.contains_entity(cube)));
    drop(command);
    assert_eq!(
        drops.load(Ordering::SeqCst),
        0,
        "the restored World now owns the payload"
    );
    drop(unavailable);
    drop(context);
    drop(level);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
