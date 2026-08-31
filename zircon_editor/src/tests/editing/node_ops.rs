use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use zircon_runtime::scene::components::{Name, NodeKind};
use zircon_runtime::scene::{DefaultLevelManager, LevelSystem, SceneError, World};
use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    CommandEffect, EditCommand, EditCommandError, EditorTransactionEngine, HistoryContextId,
    SelectionSnapshot,
};
use crate::core::editing::selection::SceneSelection;
use crate::core::gateway::{
    EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError, InProcessGateway,
};
use crate::scene::selection::WorldDomain;
use crate::ui::workbench::state::EditorStateOperationError;

use crate::core::editing::intent::EditorIntent;

use super::support::{cube_and_camera, cube_id, test_state};

const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";

struct CallbackThenErrorGateway {
    level: LevelSystem,
    fail_next_world_write: AtomicBool,
}

impl CallbackThenErrorGateway {
    fn new(level: LevelSystem) -> Self {
        Self {
            level,
            fail_next_world_write: AtomicBool::new(true),
        }
    }
}

impl EditorRuntimeGateway for CallbackThenErrorGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::invalid()
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        zircon_runtime_interface::GatewaySessionIdentity::detached()
    }

    fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        self.level.with_world(read);
        Ok(())
    }

    fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        self.level.with_world_mut(write);
        if self.fail_next_world_write.swap(false, Ordering::AcqRel) {
            Err(GatewayError::Protocol {
                message: "gateway reported after executing a world write".to_owned(),
            })
        } else {
            Ok(())
        }
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

#[test]
fn create_apply_is_applied_when_gateway_fails_after_the_callback() {
    let level = DefaultLevelManager::default().create_default_level();
    let initial_node_count = level.with_world(|scene| scene.nodes().len());
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway.clone());
    gateway
        .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
        .expect("replace gateway with callback-then-error fixture");
    let mut command = EditorCommand::create_node(NodeKind::Cube);

    let error = command
        .apply(&mut context)
        .expect_err("gateway error after create must surface");
    assert_eq!(error.effect, CommandEffect::Applied);
    let EditCommandError::ExternalEffect { source } = error.source else {
        panic!("post-callback gateway error must remain typed");
    };
    assert_eq!(
        source.downcast_ref::<GatewayError>(),
        Some(&GatewayError::Protocol {
            message: "gateway reported after executing a world write".to_owned(),
        })
    );
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count + 1,
        "the callback mutation must be visible before transaction recovery"
    );

    command
        .revert(&mut context)
        .expect("the retained create record must support recovery");
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count
    );
}

#[test]
fn play_history_routes_scene_commands_to_the_exact_play_gateway() {
    let authoring_level = DefaultLevelManager::default().create_default_level();
    let play_level = DefaultLevelManager::default().create_default_level();
    let authoring_nodes = authoring_level.with_world(|scene| scene.nodes().len());
    let play_nodes = play_level.with_world(|scene| scene.nodes().len());
    let authoring_gateway = EditorRuntimeGatewayHandle::new(Arc::new(
        InProcessGateway::for_authoring_level(authoring_level.clone()),
    ));
    let play_gateway = EditorRuntimeGatewayHandle::detached();
    let instance = crate::core::play::PlayInstanceId::for_test(47);
    play_gateway
        .replace_for_play(
            Arc::new(InProcessGateway::for_authoring_level(play_level.clone())),
            Some(instance.raw()),
        )
        .unwrap();
    let engine = EditorTransactionEngine::new(CoreEditContext::with_world_gateways(
        authoring_gateway,
        play_gateway.clone(),
    ));
    let history = HistoryContextId::PlaySession(instance);

    let mut scope = engine.begin("create play node", history).unwrap();
    scope
        .push(EditorCommand::create_node(NodeKind::Cube))
        .unwrap();
    scope.commit().unwrap();

    assert_eq!(
        authoring_level.with_world(|scene| scene.nodes().len()),
        authoring_nodes,
        "a Play transaction must not mutate the authoring world"
    );
    assert_eq!(
        play_level.with_world(|scene| scene.nodes().len()),
        play_nodes + 1
    );
    assert!(engine.undo(history).unwrap());
    assert_eq!(
        play_level.with_world(|scene| scene.nodes().len()),
        play_nodes
    );
    assert!(engine.redo(history).unwrap());

    let replacement_level = DefaultLevelManager::default().create_default_level();
    let replacement_nodes = replacement_level.with_world(|scene| scene.nodes().len());
    play_gateway
        .replace_for_play(
            Arc::new(InProcessGateway::for_authoring_level(
                replacement_level.clone(),
            )),
            Some(instance.raw()),
        )
        .unwrap();
    assert!(matches!(
        engine.undo(history),
        Err(EditCommandError::WorldRouteStale {
            world_domain: crate::core::play::WorldDomain::Play(found),
        }) if found == instance
    ));
    assert_eq!(
        play_level.with_world(|scene| scene.nodes().len()),
        play_nodes + 1,
        "the historical command remains applied in its original world"
    );
    assert_eq!(
        replacement_level.with_world(|scene| scene.nodes().len()),
        replacement_nodes,
        "a stale history must not mutate the replacement play world"
    );
    assert_eq!(
        authoring_level.with_world(|scene| scene.nodes().len()),
        authoring_nodes
    );
}

#[test]
fn create_apply_is_applied_when_selection_generation_is_exhausted() {
    let level = DefaultLevelManager::default().create_default_level();
    let initial_node_count = level.with_world(|scene| scene.nodes().len());
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway);
    context
        .bind_scene(level.clone(), SceneSelection::new(Vec::new(), None))
        .expect("bind default level to editor context");
    context
        .restore_selection_snapshot(&SelectionSnapshot::scene(
            u64::MAX,
            SceneSelection::new(Vec::new(), None),
        ))
        .expect("restore maximal selection generation");
    let mut command = EditorCommand::create_node(NodeKind::Cube);

    let error = command
        .apply(&mut context)
        .expect_err("selection failure after create must surface");
    assert_eq!(error.effect, CommandEffect::Applied);
    assert!(matches!(
        error.source,
        EditCommandError::SelectionGenerationExhausted
    ));
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count + 1,
        "the scene mutation must be visible before transaction recovery"
    );

    command
        .revert(&mut context)
        .expect("the retained create record must support recovery");
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count
    );
}

#[test]
fn create_redo_is_applied_when_selection_generation_is_exhausted() {
    let level = DefaultLevelManager::default().create_default_level();
    let initial_node_count = level.with_world(|scene| scene.nodes().len());
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway);
    context
        .bind_scene(level.clone(), SceneSelection::new(Vec::new(), None))
        .expect("bind default level to editor context");
    let mut command = EditorCommand::create_node(NodeKind::Cube);
    command
        .apply(&mut context)
        .expect("initial create must capture the retained record");
    command
        .revert(&mut context)
        .expect("initial create must be reversible before redo");
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count
    );
    context
        .restore_selection_snapshot(&SelectionSnapshot::scene(
            u64::MAX,
            SceneSelection::new(Vec::new(), None),
        ))
        .expect("restore maximal selection generation");

    let error = command
        .apply(&mut context)
        .expect_err("selection failure after create redo must surface");
    assert_eq!(error.effect, CommandEffect::Applied);
    assert!(matches!(
        error.source,
        EditCommandError::SelectionGenerationExhausted
    ));
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count + 1,
        "the retained record must be reinserted before transaction recovery"
    );

    command
        .revert(&mut context)
        .expect("the retained create record must remain reversible after redo failure");
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count
    );
}

#[test]
fn update_apply_is_applied_when_gateway_fails_after_the_callback() {
    let level = DefaultLevelManager::default().create_default_level();
    let (cube, original_name) = level.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| (node.id, node.name.clone()))
            .expect("default editor level must contain a cube")
    });
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway.clone());
    gateway
        .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
        .expect("replace gateway with callback-then-error fixture");
    let mut command = level
        .with_world(|scene| EditorCommand::rename_node(scene, cube, "Gateway Updated".to_owned()))
        .expect("capture cube rename command")
        .expect("renaming to a distinct name must create a command");

    let error = command
        .apply(&mut context)
        .expect_err("gateway error after update must surface");
    assert_eq!(error.effect, CommandEffect::Applied);
    assert_eq!(
        level.with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        "Gateway Updated",
        "the callback mutation must be visible before transaction recovery"
    );

    command
        .revert(&mut context)
        .expect("the update command must remain reversible after recovery");
    assert_eq!(
        level.with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );
}

#[test]
fn reflected_undo_compensates_when_gateway_errors_after_successful_restore() {
    let level = DefaultLevelManager::default().create_default_level();
    let (cube, original_name) = level.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| (node.id, node.name.clone()))
            .expect("default editor level must contain a cube")
    });
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway.clone());
    context
        .bind_scene(level.clone(), SceneSelection::new(vec![cube], Some(cube)))
        .expect("bind default level to editor context");
    let engine = EditorTransactionEngine::new(context);
    let command = level
        .with_world(|scene| {
            EditorCommand::set_reflected_scene_field(
                scene,
                cube,
                NAME_TYPE_PATH,
                "value",
                ReflectedValue::String("Gateway Reflected".to_owned()),
            )
        })
        .expect("capture reflected name command")
        .expect("a distinct reflected value must create a command");
    let mut scope = engine
        .begin("set reflected name", HistoryContextId::Global)
        .expect("begin reflected transaction");
    scope.push(command).expect("apply reflected command");
    scope.commit().expect("commit reflected command");

    gateway
        .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
        .expect("replace gateway with callback-then-error fixture");

    let error = engine
        .undo(HistoryContextId::Global)
        .expect_err("gateway error after reflected restore must surface");
    let EditCommandError::ExternalEffect { source } = error else {
        panic!("gateway error after reflected restore must remain typed");
    };
    assert_eq!(
        source.downcast_ref::<GatewayError>(),
        Some(&GatewayError::Protocol {
            message: "gateway reported after executing a world write".to_owned(),
        })
    );
    assert_eq!(
        level.with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        "Gateway Reflected",
        "the transaction recovery must reapply the reflected write"
    );

    assert!(
        engine.undo(HistoryContextId::Global).unwrap(),
        "the compensated reflected command must remain undoable"
    );
    assert_eq!(
        level.with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );
}

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
    state.world.expect_with_world_mut(|scene| {
        scene
            .spawn_at(cube, (Name("conflicting replacement".to_owned()),))
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
fn delete_undo_compensates_when_gateway_errors_after_successful_restore() {
    let level = DefaultLevelManager::default().create_default_level();
    let cube = level.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| node.id)
            .expect("default editor level must contain a cube")
    });
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway.clone());
    context
        .bind_scene(level.clone(), SceneSelection::new(vec![cube], Some(cube)))
        .expect("bind default level to editor context");
    let engine = EditorTransactionEngine::new(context);
    let delete = level
        .with_world(|scene| EditorCommand::delete_node(scene, cube))
        .expect("capture cube deletion command");
    let mut scope = engine
        .begin("delete cube", HistoryContextId::Global)
        .expect("begin deletion transaction");
    scope.push(delete).expect("apply cube deletion");
    scope.commit().expect("commit cube deletion");
    assert!(level.with_world(|scene| !scene.contains_entity(cube)));

    gateway
        .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
        .expect("replace gateway with callback-then-error fixture");

    let error = engine
        .undo(HistoryContextId::Global)
        .expect_err("gateway error after restore must surface");
    let EditCommandError::ExternalEffect { source } = error else {
        panic!("gateway error after restore must remain typed");
    };
    assert_eq!(
        source.downcast_ref::<GatewayError>(),
        Some(&GatewayError::Protocol {
            message: "gateway reported after executing a world write".to_owned(),
        })
    );
    assert!(
        level.with_world(|scene| !scene.contains_entity(cube)),
        "the transaction recovery must reapply the delete after a post-callback error"
    );

    assert!(
        engine.undo(HistoryContextId::Global).unwrap(),
        "the compensated delete must retain its batch for a later undo retry"
    );
    assert!(level.with_world(|scene| scene.contains_entity(cube)));
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

#[test]
fn rename_and_reparent_are_undoable() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);

    assert!(state
        .apply_intent(EditorIntent::RenameNode(cube, "Hero Cube".to_string()))
        .unwrap());
    assert!(state
        .apply_intent(EditorIntent::SetParent(cube, Some(camera)))
        .unwrap());

    state.world.expect_with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.name, "Hero Cube");
        assert_eq!(node.parent, Some(camera));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());

    state.world.expect_with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_ne!(node.name, "Hero Cube");
        assert_eq!(node.parent, None);
    });
}

#[test]
fn reparenting_multiple_nodes_commits_and_undoes_as_one_transaction() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let second_cube = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });

    assert!(state
        .apply_intent(EditorIntent::SetParents(
            vec![cube, second_cube],
            Some(camera),
        ))
        .unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().parent, Some(camera));
        assert_eq!(scene.find_node(second_cube).unwrap().parent, Some(camera));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().parent, None);
        assert_eq!(scene.find_node(second_cube).unwrap().parent, None);
    });
}

#[test]
fn reparenting_selected_parent_and_child_preserves_the_subtree() {
    let mut state = test_state();
    let (parent, new_parent) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let child = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != parent)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state
        .apply_intent(EditorIntent::SetParent(child, Some(parent)))
        .unwrap());

    assert!(state
        .apply_intent(EditorIntent::SetParents(
            vec![parent, child],
            Some(new_parent),
        ))
        .unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(parent).unwrap().parent, Some(new_parent));
        assert_eq!(scene.find_node(child).unwrap().parent, Some(parent));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(parent).unwrap().parent, None);
        assert_eq!(scene.find_node(child).unwrap().parent, Some(parent));
    });
}

#[test]
fn reparenting_multiple_nodes_cancels_the_whole_transaction_on_a_cycle() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let second_cube = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state
        .apply_intent(EditorIntent::SetParent(cube, Some(camera)))
        .unwrap());

    let error = state
        .apply_intent(EditorIntent::SetParents(
            vec![second_cube, camera],
            Some(cube),
        ))
        .unwrap_err();

    assert!(matches!(
        &error,
        EditorStateOperationError::EditCommand(EditCommandError::ExternalEffect { source })
            if source.to_string().contains("cycle")
    ));
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().parent, Some(camera));
        assert_eq!(scene.find_node(camera).unwrap().parent, None);
        assert_eq!(scene.find_node(second_cube).unwrap().parent, None);
    });
}

#[test]
fn deleting_multiple_selected_nodes_commits_and_undoes_as_one_transaction() {
    let mut state = test_state();
    let (cube, _camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let second_cube = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, second_cube],
        Some(second_cube),
    ));

    assert!(state.delete_selected().unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_none());
        assert!(scene.find_node(second_cube).is_none());
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(second_cube).is_some());
    });
}

#[test]
fn deleting_multiple_nodes_restores_the_selection_snapshot_on_undo() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let second_cube = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, second_cube],
        Some(second_cube),
    ));

    assert!(state.delete_selected().unwrap());
    assert_eq!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [camera]
    );
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(camera)
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [cube, second_cube]
    );
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(second_cube)
    );
}

#[test]
fn deleting_selection_with_the_last_camera_cancels_the_whole_transaction() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, camera],
        Some(cube),
    ));

    let error = state.delete_selected().unwrap_err();

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

#[test]
fn deleting_multiple_cameras_cancels_the_whole_transaction() {
    let mut state = test_state();
    let (cube, first_camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Camera))
        .unwrap());
    let second_camera = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Camera) && node.id != first_camera)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, first_camera, second_camera],
        Some(cube),
    ));

    let error = state.delete_selected().unwrap_err();

    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::InvariantViolation {
            invariant: "cannot delete the last remaining camera"
        })
    ));
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(first_camera).is_some());
        assert!(scene.find_node(second_camera).is_some());
    });
}

#[test]
fn deleting_selected_parent_and_child_collapses_to_one_subtree_command() {
    let mut state = test_state();
    let (cube, _camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let child = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state
        .apply_intent(EditorIntent::SetParent(child, Some(cube)))
        .unwrap());
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, child],
        Some(cube),
    ));

    assert!(state.delete_selected().unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_none());
        assert!(scene.find_node(child).is_none());
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(child).unwrap().parent, Some(cube));
    });
}
