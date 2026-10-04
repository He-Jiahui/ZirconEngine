//! 为编辑上下文和视口测试建立共享场景与节点句柄；调用方依赖稳定的测试世界，而非各自复制启动前提。
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::math::UVec2;

use crate::ui::workbench::state::EditorState;

pub(super) fn test_state() -> EditorState {
    let manager = DefaultLevelManager::default();
    let mut state =
        EditorState::with_default_selection(manager.create_default_level(), UVec2::new(1280, 720));
    state.mark_project_open();
    state
}

pub(super) fn cube_id(state: &EditorState) -> NodeId {
    state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| node.id)
            .unwrap()
    })
}

pub(super) fn cube_and_camera(state: &EditorState) -> (NodeId, NodeId) {
    state.world.expect_with_world(|scene| {
        let cube = scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| node.id)
            .unwrap();
        (cube, scene.active_camera())
    })
}
