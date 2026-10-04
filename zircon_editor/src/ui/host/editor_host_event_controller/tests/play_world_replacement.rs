use std::sync::Arc;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::{DefaultLevelManager, LevelSystem, NodeKind, World};
use zircon_runtime_interface::math::UVec2;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::engine::HistoryContextId;
use crate::core::gateway::InProcessGateway;
use crate::core::play::PlayKind;
use crate::scene::selection::SelectionMutation;
use crate::ui::host::EditorManager;
use crate::ui::workbench::state::EditorState;

use super::*;

fn active_simulate_controller() -> (EditorHostEventController, PlayInstanceId, LevelSystem) {
    let core = CoreRuntime::new();
    let manager = Arc::new(EditorManager::new(&core.handle()).expect("editor manager"));
    let state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
        Arc::clone(manager.context()),
    );
    let controller = EditorHostEventController::new(state, manager);
    let play_level = DefaultLevelManager::default().create_default_level();
    let instance = controller
        .start_test_play_gateway(
            PlayKind::Simulate,
            Arc::new(InProcessGateway::for_authoring_level(play_level.clone())),
        )
        .expect("play gateway");
    controller
        .shell()
        .lock()
        .state
        .enter_play_mode()
        .expect("editor play state");
    assert!(controller.sync_active_selection_world_domain());
    (controller, instance, play_level)
}

#[test]
fn replacement_retires_the_matching_play_selection() {
    let (controller, instance, play_level) = active_simulate_controller();
    let history = HistoryContextId::PlaySession(instance);
    let mut scope = controller
        .context()
        .transactions()
        .begin("create runtime node", history)
        .expect("attached Play history");
    scope
        .push(EditorCommand::create_node(NodeKind::Cube))
        .expect("runtime command");
    scope.commit().expect("runtime command commit");
    assert_eq!(
        controller.apply_play_viewport_pick_selection(
            instance,
            Some(991),
            SelectionMutation::Replace,
        ),
        Some(true)
    );
    let identity = controller
        .world_gateway_identity(WorldDomain::Play(instance))
        .expect("attached identity");
    play_level.replace_world_and_reset_runtime_state(World::empty());
    let replacement_epoch = play_level.capture_world_replacement_epoch();

    let report = controller
        .retire_replaced_play_world(instance, &identity, replacement_epoch)
        .expect("replacement retirement");

    assert_eq!(report.replacement_epoch(), replacement_epoch);
    assert!(report.history_discarded());
    assert!(report.selection_cleared());
    assert_eq!(
        controller
            .context()
            .transactions()
            .history_status(history)
            .unwrap()
            .len,
        0
    );
    assert!(controller
        .gateway_for(WorldDomain::Play(instance))
        .is_some());
    assert_eq!(
        controller
            .shell()
            .lock()
            .state
            .viewport_controller
            .selection()
            .active_primary(),
        None
    );
}

#[test]
fn replacement_from_a_stale_gateway_cannot_retire_current_play_state() {
    let (controller, instance, _play_level) = active_simulate_controller();
    controller.apply_play_viewport_pick_selection(instance, Some(991), SelectionMutation::Replace);
    let stale = GatewaySessionIdentity::new(
        99,
        zircon_runtime_interface::ZrRuntimeSessionHandle::new(99),
        99,
        None,
    );

    assert!(controller
        .retire_replaced_play_world(instance, &stale, 2)
        .is_err());
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
}
