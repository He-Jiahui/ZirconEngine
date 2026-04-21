use zircon_runtime::core::math::UVec2;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime::scene::NodeId;

use crate::ui::workbench::state::EditorState;

pub(super) fn test_state() -> EditorState {
    let manager = DefaultLevelManager::default();
    EditorState::with_default_selection(manager.create_default_level(), UVec2::new(1280, 720))
}

pub(super) fn cube_id(state: &EditorState) -> NodeId {
    state.world.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| node.id)
            .unwrap()
    })
}

pub(super) fn cube_and_camera(state: &EditorState) -> (NodeId, NodeId) {
    state.world.with_world(|scene| {
        let cube = scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| node.id)
            .unwrap();
        (cube, scene.active_camera())
    })
}
