use super::*;
use crate::core::editing::command::EditorCommand;
use crate::core::editing::intent::EditorIntent;
use crate::core::editor_message::DocumentId;
use zircon_runtime::scene::{components::NodeKind, DefaultLevelManager, NodeId};
use zircon_runtime_interface::math::UVec2;

fn state() -> EditorState {
    let mut state = EditorState::with_default_selection(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
    );
    state.mark_project_open();
    state
}

fn cube(state: &EditorState) -> NodeId {
    state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id
    })
}

fn rename(state: &mut EditorState, name: &str) {
    let node = cube(state);
    let command = state.world.expect_with_world(|scene| {
        EditorCommand::rename_node(scene, node, name.to_owned())
            .unwrap()
            .unwrap()
    });
    state
        .execute_scene_command("reload fixture rename", command)
        .unwrap();
}

fn name(state: &EditorState) -> String {
    let node = cube(state);
    state
        .world
        .expect_with_world(|scene| scene.find_node(node).unwrap().name.clone())
}

fn authorization(state: &EditorState) -> SceneReloadDiscardAuthorization {
    SceneReloadDiscardAuthorization::capture(
        state.transactions(),
        state.scene_history_context().unwrap(),
    )
    .unwrap()
    .unwrap()
}

#[test]
fn stale_discard_generation_preserves_world_selection_and_undo() {
    let mut state = state();
    rename(&mut state, "authorized edit");
    let token = authorization(&state);
    rename(&mut state, "edit while loading");
    let history = state.scene_history_context().unwrap();
    let before = state.transactions().history_status(history).unwrap();
    let selection = state
        .transactions()
        .with_context::<CoreEditContext, _>(CoreEditContext::selection_snapshot)
        .unwrap()
        .unwrap();

    assert_eq!(
        state.reload_active_scene_world(
            &mut Some(DefaultLevelManager::default().create_default_level().into()),
            Some(token),
        ),
        Err(EditorStateOperationError::SceneTransitionDirty),
    );
    assert_eq!(name(&state), "edit while loading");
    assert_eq!(
        state
            .transactions()
            .with_context::<CoreEditContext, _>(CoreEditContext::selection_snapshot)
            .unwrap()
            .unwrap(),
        selection
    );
    assert_eq!(
        state.transactions().history_status(history).unwrap(),
        before
    );
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(name(&state), "authorized edit");
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(name(&state), "edit while loading");
}

#[test]
fn unchanged_discard_generation_reloads_and_clears_only_the_authorized_history() {
    let mut state = state();
    let original = name(&state);
    rename(&mut state, "authorized edit");
    let token = authorization(&state);
    let history = state.scene_history_context().unwrap();

    state
        .reload_active_scene_world(
            &mut Some(DefaultLevelManager::default().create_default_level().into()),
            Some(token),
        )
        .unwrap();

    assert_eq!(name(&state), original);
    assert_eq!(state.scene_history_context().unwrap(), history);
    let status = state.transactions().history_status(history).unwrap();
    assert!(!status.dirty);
    assert!(!status.can_undo);
}

#[test]
fn clean_reload_rechecks_dirty_history_when_the_world_is_installed() {
    let mut state = state();
    state.prepare_scene_transition().unwrap();
    rename(&mut state, "edit after prepare");
    let history = state.scene_history_context().unwrap();
    let before = state.transactions().history_status(history).unwrap();

    assert_eq!(
        state.reload_active_scene_world(
            &mut Some(DefaultLevelManager::default().create_default_level().into()),
            None,
        ),
        Err(EditorStateOperationError::SceneTransitionDirty),
    );
    assert_eq!(name(&state), "edit after prepare");
    assert_eq!(
        state.transactions().history_status(history).unwrap(),
        before
    );
}

#[test]
fn discard_authorization_does_not_cover_another_history_with_the_same_generation() {
    let mut state = state();
    rename(&mut state, "first document edit");
    let token = authorization(&state);
    let first_history = state.scene_history_context().unwrap();
    state.bind_scene_document(DocumentId::new(2));
    rename(&mut state, "second document edit");
    let current = authorization(&state);
    let second_history = state.scene_history_context().unwrap();
    assert_eq!(
        state
            .transactions()
            .history_generation_snapshot(first_history)
            .unwrap(),
        state
            .transactions()
            .history_generation_snapshot(second_history)
            .unwrap()
    );
    assert_ne!(first_history, second_history);
    assert_ne!(token, current);

    assert_eq!(
        state.reload_active_scene_world(
            &mut Some(DefaultLevelManager::default().create_default_level().into()),
            Some(token),
        ),
        Err(EditorStateOperationError::SceneTransitionDirty),
    );
    assert_eq!(name(&state), "second document edit");
}

#[test]
fn a_saved_scene_can_reload_without_a_discard_authorization() {
    let mut state = state();
    rename(&mut state, "saved edit");
    let history = state.scene_history_context().unwrap();
    let save = state.transactions().capture_save_token(history).unwrap();
    state
        .transactions()
        .mark_saved_if_unchanged(history, save)
        .unwrap();
    assert_eq!(
        SceneReloadDiscardAuthorization::capture(state.transactions(), history).unwrap(),
        None
    );

    state
        .reload_active_scene_world(
            &mut Some(DefaultLevelManager::default().create_default_level().into()),
            None,
        )
        .unwrap();

    assert!(!state.transactions().history_status(history).unwrap().dirty);
}

#[test]
fn saving_new_edits_does_not_expand_an_older_discard_authorization() {
    let mut state = state();
    rename(&mut state, "authorized edit");
    let token = authorization(&state);
    rename(&mut state, "new saved edit");
    let history = state.scene_history_context().unwrap();
    let save = state.transactions().capture_save_token(history).unwrap();
    state
        .transactions()
        .mark_saved_if_unchanged(history, save)
        .unwrap();
    let before = state.transactions().history_status(history).unwrap();
    assert!(!before.dirty);

    assert_eq!(
        state.reload_active_scene_world(
            &mut Some(DefaultLevelManager::default().create_default_level().into()),
            Some(token),
        ),
        Err(EditorStateOperationError::SceneTransitionDirty),
    );
    assert_eq!(name(&state), "new saved edit");
    assert_eq!(
        state.transactions().history_status(history).unwrap(),
        before
    );
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(name(&state), "authorized edit");
}

#[test]
fn playing_with_edit_selection_defers_before_selecting_a_history_context() {
    let mut state = state();
    rename(&mut state, "dirty authoring edit");
    let history = state.scene_history_context().unwrap();
    let before = state.transactions().history_status(history).unwrap();
    state.enter_play_mode().unwrap();
    assert!(state.is_playing());
    assert_eq!(
        state.viewport_controller.selection().active_domain(),
        crate::core::play::WorldDomain::Edit
    );
    let mut prepared = Some(DefaultLevelManager::default().create_default_level().into());

    assert_eq!(
        state.reload_active_scene_world(&mut prepared, None),
        Err(EditorStateOperationError::SceneEditingDisabledDuringPlay),
    );
    assert!(prepared.is_some());
    assert_eq!(name(&state), "dirty authoring edit");
    assert_eq!(
        state.transactions().history_status(history).unwrap(),
        before
    );
    state.exit_play_mode().unwrap();
    assert_eq!(
        state.reload_active_scene_world(&mut prepared, None),
        Err(EditorStateOperationError::SceneTransitionDirty)
    );
    assert!(prepared.is_some());
}

fn pending_rename(
    state: &mut EditorState,
    name: &str,
    mode: crate::core::editing::engine::MergeMode,
) {
    let node = cube(state);
    let command = state.world.expect_with_world(|scene| {
        EditorCommand::rename_node(scene, node, name.to_owned())
            .unwrap()
            .unwrap()
    });
    let history = state.scene_history_context().unwrap();
    state
        .transactions()
        .execute_operation(
            "reload pending rename",
            history,
            Some("reload pending rename"),
            mode,
            Box::new(command),
        )
        .unwrap();
}
#[test]
fn unchanged_pending_discard_reloads_without_self_invalidating_on_flush() {
    let mut state = state();
    let original = name(&state);
    pending_rename(
        &mut state,
        "pending authorization",
        crate::core::editing::engine::MergeMode::Ends,
    );
    let token = authorization(&state);
    let history = state.scene_history_context().unwrap();
    assert_eq!(
        state
            .transactions()
            .history_generation_snapshot(history)
            .unwrap(),
        0
    );
    let mut world = Some(DefaultLevelManager::default().create_default_level().into());
    state
        .reload_active_scene_world(&mut world, Some(token))
        .unwrap();
    assert!(world.is_none());
    assert_eq!(name(&state), original);
    assert!(!state.transactions().is_dirty(history).unwrap());
}
#[test]
fn stale_pending_discard_preserves_prepared_world_new_edits_and_uncommitted_group() {
    for mode in [
        crate::core::editing::engine::MergeMode::Disable,
        crate::core::editing::engine::MergeMode::Ends,
    ] {
        let mut state = state();
        pending_rename(&mut state, "pending authorization", mode);
        let token = authorization(&state);
        pending_rename(&mut state, "newer pending edit", mode);
        let history = state.scene_history_context().unwrap();
        let mut world = Some(DefaultLevelManager::default().create_default_level().into());
        assert_eq!(
            state.reload_active_scene_world(&mut world, Some(token)),
            Err(EditorStateOperationError::SceneTransitionDirty)
        );
        assert!(world.is_some());
        assert_eq!(name(&state), "newer pending edit");
        assert_eq!(
            state
                .transactions()
                .history_generation_snapshot(history)
                .unwrap(),
            0
        );
        assert_eq!(state.transactions().history_status(history).unwrap().len, 0);
        assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    }
}

#[test]
fn reload_two_step_exhaustion_preserves_pending_world_selection_seed_and_decision() {
    for dirty_exhaustion in [false, true] {
        let mut state = state();
        pending_rename(
            &mut state,
            "pending budget edit",
            crate::core::editing::engine::MergeMode::Ends,
        );
        let history = state.scene_history_context().unwrap();
        if dirty_exhaustion {
            state
                .transactions()
                .set_dirty_generation_for_test(u64::MAX - 1);
        } else {
            state
                .transactions()
                .set_history_generation_for_test(history, u64::MAX - 1);
        }
        let token = authorization(&state);
        let selection = state.viewport_controller.selection().clone();
        let mut prepared = Some(DefaultLevelManager::default().create_default_level().into());
        let result = state.reload_active_scene_world(&mut prepared, Some(token.clone()));
        assert!(matches!(
            result,
            Err(EditorStateOperationError::EditCommand(
                EditCommandError::HistoryGenerationExhausted { .. }
                    | EditCommandError::HistoryDirtyGenerationExhausted
            ))
        ));
        assert!(prepared.is_some());
        assert_eq!(name(&state), "pending budget edit");
        assert_eq!(state.viewport_controller.selection(), &selection);
        assert_eq!(authorization(&state), token);
        assert_eq!(state.transactions().history_status(history).unwrap().len, 0);
        state.transactions().set_dirty_generation_for_test(0);
        state
            .transactions()
            .set_history_generation_for_test(history, 0);
        assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    }
}
#[test]
fn reload_selection_preparation_exhaustion_preserves_pending_world_and_seed() {
    let mut state = state();
    state
        .transactions()
        .with_context_mut::<CoreEditContext, _>(|context| {
            context.set_selection_generation_for_test(u64::MAX)
        })
        .unwrap();
    pending_rename(
        &mut state,
        "pending selection budget edit",
        crate::core::editing::engine::MergeMode::Ends,
    );
    let history = state.scene_history_context().unwrap();
    let token = authorization(&state);
    let mut prepared = Some(DefaultLevelManager::default().create_default_level().into());
    assert!(matches!(
        state.reload_active_scene_world(&mut prepared, Some(token.clone())),
        Err(EditorStateOperationError::EditCommand(
            EditCommandError::SelectionGenerationExhausted
        ))
    ));
    assert!(prepared.is_some());
    assert_eq!(name(&state), "pending selection budget edit");
    assert_eq!(authorization(&state), token);
    assert_eq!(state.transactions().history_status(history).unwrap().len, 0);
}
