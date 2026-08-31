use zircon_runtime::scene::NodeId;

use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventSource, SelectionHostEvent,
};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_hierarchy_selection(
    runtime: &EditorHostEventController,
    node_id: NodeId,
) -> Result<UiHostEventEffects, String> {
    let world_domain = runtime.active_hierarchy_world_domain();
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::RetainedHost,
            EditorEvent::Selection(SelectionHostEvent::SelectSceneNode {
                world_domain,
                node_id,
            }),
        ),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime::core::CoreRuntime;
    use zircon_runtime::scene::DefaultLevelManager;
    use zircon_runtime_interface::math::UVec2;

    use crate::core::gateway::DetachedEditorRuntimeGateway;
    use crate::core::play::{PlayKind, WorldDomain};
    use crate::ui::host::EditorManager;
    use crate::ui::workbench::state::EditorState;

    use super::*;

    #[test]
    fn hierarchy_selection_targets_the_active_play_world_without_touching_edit_selection() {
        let core = CoreRuntime::new();
        let manager = Arc::new(EditorManager::new(&core.handle()).expect("editor manager"));
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let runtime = EditorHostEventController::new(state, manager);
        let edit_primary = runtime
            .shell()
            .lock()
            .state
            .viewport_controller
            .selection()
            .active_primary();
        let instance = runtime
            .start_test_play_gateway(PlayKind::Simulate, Arc::new(DetachedEditorRuntimeGateway))
            .expect("play gateway");
        runtime
            .shell()
            .lock()
            .state
            .enter_play_mode()
            .expect("editor play state");
        assert!(runtime.sync_active_selection_world_domain());

        dispatch_hierarchy_selection(&runtime, 991).expect("play hierarchy selection");

        let shell = runtime.shell().lock();
        let selection = shell.state.viewport_controller.selection();
        assert_eq!(selection.active_domain(), WorldDomain::Play(instance));
        assert_eq!(selection.active_primary(), Some(991));
        assert_eq!(selection.primary(WorldDomain::Edit), edit_primary);
    }
}
