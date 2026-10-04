use std::sync::Arc;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;

use crate::core::gateway::DetachedEditorRuntimeGateway;
use crate::core::play::PlayKind;
use crate::scene::selection::SelectionMutation;
use crate::ui::host::EditorManager;
use crate::ui::workbench::state::EditorState;

use super::*;

#[test]
fn renderer_pick_changes_only_the_matching_play_selection_domain() {
    let core = CoreRuntime::new();
    let manager = Arc::new(EditorManager::new(&core.handle()).expect("editor manager"));
    let state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
        Arc::clone(manager.context()),
    );
    let controller = EditorHostEventController::new(state, manager);
    let edit_primary = controller
        .shell()
        .lock()
        .state
        .viewport_controller
        .selection()
        .active_primary();
    let instance = controller
        .start_test_play_gateway(PlayKind::Simulate, Arc::new(DetachedEditorRuntimeGateway))
        .expect("play gateway");
    controller
        .shell()
        .lock()
        .state
        .enter_play_mode()
        .expect("editor play state");
    assert!(controller.sync_active_selection_world_domain());

    assert_eq!(
        controller.apply_play_viewport_pick_selection(
            instance,
            Some(991),
            SelectionMutation::Replace,
        ),
        Some(true)
    );
    assert_eq!(
        controller
            .shell()
            .lock()
            .state
            .viewport_controller
            .selection()
            .active_primary(),
        Some(991)
    );

    controller
        .shell()
        .lock()
        .state
        .exit_play_mode()
        .expect("restore edit selection");
    assert_eq!(
        controller
            .shell()
            .lock()
            .state
            .viewport_controller
            .selection()
            .active_primary(),
        edit_primary
    );
}
