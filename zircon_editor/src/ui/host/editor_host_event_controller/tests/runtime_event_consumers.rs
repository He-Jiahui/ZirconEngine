use std::sync::Arc;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;

use crate::core::gateway::DetachedEditorRuntimeGateway;
use crate::core::play::PlayKind;
use crate::ui::workbench::state::EditorState;

use super::EditorHostEventController;
use crate::ui::host::EditorManager;

#[test]
fn play_consumer_start_uses_capabilities_from_an_already_locked_shell() {
    let core = CoreRuntime::new();
    let manager =
        Arc::new(EditorManager::new(&core.handle()).expect("the test manager should initialize"));
    let state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
        Arc::clone(manager.context()),
    );
    let controller = EditorHostEventController::new(state, manager);
    controller
        .start_test_play_gateway(PlayKind::Play, Arc::new(DetachedEditorRuntimeGateway))
        .expect("the test play gateway should attach");

    let shell = controller.shell().lock();
    let enabled_capabilities = shell
        .manager
        .capability_snapshot()
        .enabled_capabilities()
        .to_vec();
    controller
        .begin_runtime_event_consumers_with_capabilities(&enabled_capabilities)
        .expect("consumer startup should not reacquire the held workbench shell mutex");
    controller
        .end_runtime_event_consumers()
        .expect("the test consumer session should retire cleanly");
}
