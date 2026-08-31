use std::sync::Arc;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::engine::HistoryContextId;
use crate::core::editing::intent::EditorIntent;
use crate::core::gateway::InProcessGateway;
use crate::core::play::PlayInstanceId;
use crate::scene::selection::WorldDomain;
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::state::{EditorState, EditorStateOperationError, KeepPlayChangesError};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{DefaultLevelManager, DynamicScene, World};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::super::support::{cube_and_camera, cube_id, test_state};
use super::viewport::begin_moved_gizmo_drag;

fn play_instance() -> PlayInstanceId {
    PlayInstanceId::for_test(1)
}

fn play_domain() -> WorldDomain {
    WorldDomain::Play(play_instance())
}

#[test]
fn authoring_selection_intent_cannot_write_the_active_play_selection_domain() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state.apply_intent(EditorIntent::SelectNode(cube)).unwrap());
    assert!(state.enter_play_mode().unwrap());
    assert!(state.activate_play_selection_domain(play_instance()));

    let error = state
        .apply_intent(EditorIntent::SelectNode(camera))
        .expect_err("an authoring selection intent must remain world-qualified");

    assert_eq!(
        error,
        EditorStateOperationError::SelectionWorldMismatch {
            requested: WorldDomain::Edit,
            active: play_domain(),
        }
    );
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(cube)
    );
    assert_eq!(
        state
            .viewport_controller
            .selection()
            .primary(WorldDomain::Edit),
        Some(cube)
    );
}

#[test]
fn play_mode_restores_edit_world_and_history_on_exit() {
    let manager = DefaultLevelManager::default();
    let mut state = EditorState::project(
        manager.create_default_level(),
        UVec2::new(1280, 720),
        "sandbox-project",
    );
    let cube = cube_id(&state);
    let original_name = state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone());

    assert!(state.apply_intent(EditorIntent::SelectNode(cube)).unwrap());
    assert!(state
        .apply_intent(EditorIntent::RenameNode(cube, "Edited Cube".to_string()))
        .unwrap());
    let edit_world = state.world.expect_with_world(|scene| scene.clone());
    assert!(state.snapshot().can_undo);

    assert!(state.enter_play_mode().unwrap());
    assert!(state.is_playing());
    assert!(state.scene_viewport_settings().gizmos_enabled);
    let play_snapshot = state.snapshot();
    assert_eq!(play_snapshot.session_mode, EditorSessionMode::Playing);
    assert!(!play_snapshot.can_undo);

    let play_edit_error = state
        .apply_intent(EditorIntent::RenameNode(cube, "Runtime Cube".to_string()))
        .expect_err("play-world changes must not enter edit history");
    assert!(matches!(
        play_edit_error,
        EditorStateOperationError::SceneEditingDisabledDuringPlay
    ));
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        "Edited Cube"
    );
    assert!(!state.snapshot().can_undo);

    assert!(state.exit_play_mode().unwrap());
    assert!(!state.is_playing());
    assert!(state.scene_viewport_settings().gizmos_enabled);
    let restored_snapshot = state.snapshot();
    assert_eq!(restored_snapshot.session_mode, EditorSessionMode::Project);
    assert!(restored_snapshot.can_undo);
    assert_eq!(
        state.world.expect_with_world(|scene| scene.clone()),
        edit_world
    );
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(cube)
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );
}

#[test]
fn play_undo_redo_routes_only_to_the_active_play_world() {
    let mut state = test_state();
    let cube = cube_id(&state);
    assert!(state
        .apply_intent(EditorIntent::RenameNode(cube, "Edited Cube".to_owned()))
        .unwrap());
    let authoring_nodes = state.world.snapshot().node_records().len();

    assert!(state.enter_play_mode().unwrap());
    assert!(!state.snapshot().can_undo);

    let instance = play_instance();
    let play_level = DefaultLevelManager::default().create_default_level();
    let play_nodes = play_level.with_world(|scene| scene.nodes().len());
    state
        .context
        .play_gateway_handle()
        .replace_for_play(
            Arc::new(InProcessGateway::for_authoring_level(play_level.clone())),
            Some(instance.raw()),
        )
        .unwrap();
    assert!(state.activate_play_selection_domain(instance));

    let history = HistoryContextId::PlaySession(instance);
    let mut scope = state
        .transactions()
        .begin("create play node", history)
        .unwrap();
    scope
        .push(EditorCommand::create_node(NodeKind::Cube))
        .unwrap();
    scope.commit().unwrap();

    assert!(state.snapshot().can_undo);
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        play_level.with_world(|scene| scene.nodes().len()),
        play_nodes
    );
    assert_eq!(state.world.snapshot().node_records().len(), authoring_nodes);
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        "Edited Cube"
    );
    assert!(state.snapshot().can_redo);

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        play_level.with_world(|scene| scene.nodes().len()),
        play_nodes + 1
    );
    assert_eq!(state.world.snapshot().node_records().len(), authoring_nodes);

    assert!(state.exit_play_mode().unwrap());
    assert!(state.snapshot().can_undo);
}

#[test]
fn play_snapshot_preserves_authoring_entity_ids_in_an_empty_runtime_world() {
    let state = test_state();
    let cube = cube_id(&state);
    let authoring = state.world.expect_with_world(|scene| scene.clone());
    let snapshot = DynamicScene::from_world(&authoring).unwrap();
    let mut play_world = World::empty();

    let remap = snapshot.spawn_into(&mut play_world).unwrap();

    assert_eq!(remap.get(cube), Some(cube));
    assert!(play_world.contains_entity(cube));
}

#[test]
fn keep_play_changes_copies_serializable_properties_as_one_authoring_transaction() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state.world.expect_with_world_mut(|scene| {
        scene.set_parent(cube, Some(camera)).unwrap();
    });
    let authoring_name = state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone());
    let play_world = state.world.expect_with_world(|scene| scene.clone());
    let play_level = DefaultLevelManager::default().create_level(play_world, Default::default());
    play_level.with_world_mut(|scene| {
        scene.rename_node(cube, "Runtime Cube").unwrap();
        scene.set_parent(cube, None).unwrap();
    });

    assert!(state.enter_play_mode().unwrap());
    let instance = play_instance();
    state
        .context
        .play_gateway_handle()
        .replace_for_play(
            Arc::new(InProcessGateway::for_authoring_level(play_level.clone())),
            Some(instance.raw()),
        )
        .unwrap();
    assert!(state.activate_play_selection_domain(instance));
    state
        .viewport_controller
        .selection_mut()
        .select_only(play_domain(), cube);

    assert!(state.keep_play_changes().unwrap());
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Document(
                crate::core::editor_message::DocumentId::new(1)
            ))
            .unwrap()
            .len,
        1
    );
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().name, "Runtime Cube");
        assert_eq!(scene.parent_of(cube), Some(camera));
    });
    assert_eq!(
        state.viewport_controller.selection().active_domain(),
        play_domain()
    );

    assert!(state.exit_play_mode().unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().name, "Runtime Cube");
        assert_eq!(scene.parent_of(cube), Some(camera));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().name, authoring_name);
        assert_eq!(scene.parent_of(cube), Some(camera));
    });
    play_level.with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().name, "Runtime Cube");
        assert_eq!(scene.parent_of(cube), None);
    });

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().name, "Runtime Cube");
        assert_eq!(scene.parent_of(cube), Some(camera));
    });
}

#[test]
fn keep_play_changes_rejects_runtime_spawned_entities_without_partial_history() {
    let mut state = test_state();
    let play_world = state.world.expect_with_world(|scene| scene.clone());
    let play_level = DefaultLevelManager::default().create_level(play_world, Default::default());
    let runtime_entity = play_level.with_world_mut(|scene| {
        scene
            .spawn_node(NodeKind::Cube)
            .expect("runtime-only entity should spawn")
    });

    assert!(state.enter_play_mode().unwrap());
    let instance = play_instance();
    state
        .context
        .play_gateway_handle()
        .replace_for_play(
            Arc::new(InProcessGateway::for_authoring_level(play_level)),
            Some(instance.raw()),
        )
        .unwrap();
    assert!(state.activate_play_selection_domain(instance));
    assert!(state
        .viewport_controller
        .selection_mut()
        .select_only(play_domain(), runtime_entity));

    let error = state.keep_play_changes().unwrap_err();

    assert!(matches!(
        error,
        EditorStateOperationError::KeepPlayChanges(
            KeepPlayChangesError::AuthoringCounterpartMissing { entity }
        ) if entity == runtime_entity
    ));
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Document(
                crate::core::editor_message::DocumentId::new(1)
            ))
            .unwrap()
            .len,
        0
    );
}

#[test]
fn import_is_rejected_during_play_without_poisoning_edit_history() {
    let mut state = test_state();
    let cube = cube_id(&state);
    let original_name = state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone());
    assert!(state
        .apply_intent(EditorIntent::RenameNode(cube, "Edited Cube".to_string()))
        .unwrap());
    let edit_node_count = state.world.snapshot().node_records().len();

    assert!(state.enter_play_mode().unwrap());
    let error = state
        .import_mesh_asset(
            ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/play-mode.obj",
            )),
            ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/play-mode.zmaterial",
            )),
            "res://models/play-mode.obj",
        )
        .expect_err("imports must not bypass play-mode scene isolation");
    assert!(matches!(
        error,
        EditorStateOperationError::SceneEditingDisabledDuringPlay
    ));
    assert_eq!(state.world.snapshot().node_records().len(), edit_node_count);

    assert!(state.exit_play_mode().unwrap());
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );
}

#[test]
fn play_mode_restores_the_complete_dual_domain_selection_model() {
    let manager = DefaultLevelManager::default();
    let mut state = EditorState::project(
        manager.create_default_level(),
        UVec2::new(1280, 720),
        "sandbox-project",
    );
    let (cube, camera) = cube_and_camera(&state);
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [camera, cube],
        Some(cube)
    ));
    let selection_before_play = state.viewport_controller.selection().clone();

    assert!(state.enter_play_mode().unwrap());
    assert!(state.activate_play_selection_domain(play_instance()));
    assert_eq!(
        state.viewport_controller.selection().active_domain(),
        play_domain()
    );
    assert_eq!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [camera, cube]
    );
    assert!(state
        .viewport_controller
        .selection_mut()
        .select_only(WorldDomain::Edit, camera));
    assert!(state
        .viewport_controller
        .selection_mut()
        .select_only(play_domain(), cube));

    assert!(state.exit_play_mode().unwrap());
    assert_eq!(
        state.viewport_controller.selection(),
        &selection_before_play
    );
}

#[test]
fn entering_play_during_gizmo_drag_restores_the_edit_transform() {
    let mut state = test_state();
    let (cube, initial) = begin_moved_gizmo_drag(&mut state);

    assert!(state.enter_play_mode().unwrap());
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().transform),
        initial
    );
    assert!(!state.viewport_controller.is_handle_drag_active());
    assert!(state.exit_play_mode().unwrap());
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().transform),
        initial
    );
}

#[test]
fn play_mode_rejects_unloaded_welcome_world() {
    let mut state = EditorState::welcome(UVec2::new(1280, 720), WelcomePaneSnapshot::default());

    let error = state
        .enter_play_mode()
        .expect_err("welcome mode has no world to play");

    assert!(matches!(error, EditorStateOperationError::NoProjectOpen));
    assert_eq!(state.snapshot().status_line, "No project open");

    let exit_error = state
        .exit_play_mode()
        .expect_err("welcome mode has no world to restore");
    assert!(matches!(
        exit_error,
        EditorStateOperationError::NoProjectOpen
    ));
    assert!(!state.is_playing());
    let snapshot = state.snapshot();
    assert_eq!(snapshot.session_mode, EditorSessionMode::Welcome);
    assert!(snapshot.status_line.contains("No project open"));
}
