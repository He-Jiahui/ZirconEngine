use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::SelectionSnapshot;
use crate::core::editing::selection::SceneSelection;
use crate::ui::workbench::state::EditorState;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime::scene::NodeId;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::math::UVec2;

pub(super) fn test_state() -> EditorState {
    let manager = DefaultLevelManager::default();
    let mut state =
        EditorState::with_default_selection(manager.create_default_level(), UVec2::new(1280, 720));
    state.mark_project_open();
    state
}

pub(super) fn cube_id(state: &EditorState) -> NodeId {
    state
        .world
        .with_world(|scene: &zircon_runtime::scene::Scene| {
            scene
                .nodes()
                .iter()
                .find(|node| matches!(node.kind, NodeKind::Cube))
                .map(|node| node.id)
                .unwrap()
        })
}

pub(super) fn camera_id(state: &EditorState) -> NodeId {
    state
        .world
        .with_world(|scene: &zircon_runtime::scene::Scene| {
            scene
                .nodes()
                .iter()
                .find(|node| matches!(node.kind, NodeKind::Camera))
                .map(|node| node.id)
                .unwrap()
        })
}

pub(super) fn transaction_selection(state: &EditorState) -> SceneSelection {
    state
        .transactions()
        .with_context::<CoreEditContext, _>(CoreEditContext::scene_selection)
        .unwrap()
        .unwrap()
        .unwrap()
}

pub(super) fn transaction_selection_snapshot(state: &EditorState) -> SelectionSnapshot {
    state
        .transactions()
        .with_context::<CoreEditContext, _>(CoreEditContext::selection_snapshot)
        .unwrap()
        .unwrap()
}
