use std::sync::Arc;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;

use crate::core::editor_message::DocumentId;
use crate::core::gateway::DetachedEditorRuntimeGateway;
use crate::core::play::{PlayInstanceId, PlayKind};
use crate::ui::host::{EditorHostEventController, EditorManager};

use super::{EditorState, EditorStateOperationError, HistoryContextId, WorldDomain};

fn with_active_play_state(test: impl FnOnce(&mut EditorState, PlayInstanceId)) {
    let core = CoreRuntime::new();
    let manager = Arc::new(EditorManager::new(&core.handle()).expect("editor manager"));
    let state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
        Arc::clone(manager.context()),
    );
    let controller = EditorHostEventController::new(
        state,
        crate::tests::support::register_test_editor_manager(&core, manager),
    )
    .expect("editor controller manager admission");
    let instance = controller
        .start_test_play_gateway(PlayKind::Play, Arc::new(DetachedEditorRuntimeGateway))
        .expect("play gateway");
    let mut shell = controller.shell().lock();
    shell.state.enter_play_mode().expect("editor play state");
    shell.state.activate_play_selection_domain(instance);
    test(&mut shell.state, instance);
}

#[test]
fn project_save_uses_edit_document_while_commands_target_play_history() {
    with_active_play_state(|state, instance| {
        let document = DocumentId::new(7);
        state.bind_scene_document(document);

        assert_eq!(
            state.scene_history_context().expect("play command history"),
            HistoryContextId::PlaySession(instance)
        );
        assert_eq!(
            state
                .authoring_scene_history_context()
                .expect("persisted scene history"),
            HistoryContextId::Document(document)
        );
        assert!(state.is_playing());
        assert_eq!(
            state.viewport_controller.selection().active_domain(),
            WorldDomain::Play(instance)
        );
    });
}

#[test]
fn project_save_requires_an_edit_document_even_with_active_play_history() {
    with_active_play_state(|state, instance| {
        state.clear_scene_document_binding();

        assert_eq!(
            state.scene_history_context().expect("play command history"),
            HistoryContextId::PlaySession(instance)
        );
        assert!(matches!(
            state.authoring_scene_history_context(),
            Err(EditorStateOperationError::SceneDocumentNotActive)
        ));
    });
}

#[test]
fn project_save_and_commands_use_the_same_document_after_stop() {
    with_active_play_state(|state, _instance| {
        let document = DocumentId::new(7);
        state.bind_scene_document(document);
        state.exit_play_mode().expect("stop play state");

        assert_eq!(
            state
                .authoring_scene_history_context()
                .expect("persisted scene history"),
            HistoryContextId::Document(document)
        );
        assert_eq!(
            state.scene_history_context().expect("edit command history"),
            HistoryContextId::Document(document)
        );
    });
}
