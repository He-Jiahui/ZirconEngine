use std::sync::Arc;

use zircon_runtime_interface::math::UVec2;

use crate::core::context::ToolSchedulerService;
use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::settings::SettingsMutationCoordinator;
use crate::core::tools::{ToolResourceKey, ToolResourceSet};
use crate::scene::viewport::handles::HandleToolRegistry;
use crate::scene::viewport::pointer::ViewportOverlayPointerRouter;
use crate::scene::viewport::ViewportInteractionExtractCache;

use super::{
    scene_viewport_controller::SceneToolIdentity, scene_viewport_state::SceneViewportState,
    SceneViewportController,
};

impl SceneViewportController {
    #[cfg(test)]
    pub(crate) fn new(viewport_size: UVec2) -> Self {
        let settings_mutations = Arc::new(SettingsMutationCoordinator::in_memory_with_defaults());
        Self::with_settings_coordinator(viewport_size, settings_mutations)
    }

    pub(crate) fn with_settings_coordinator(
        viewport_size: UVec2,
        settings_mutations: Arc<SettingsMutationCoordinator>,
    ) -> Self {
        Self::with_settings_and_tools(
            viewport_size,
            settings_mutations,
            ToolSchedulerService::new(SharedEditorMessageBus::default()),
            ViewInstanceId::new("test.scene#1"),
        )
    }

    pub(crate) fn with_settings_and_tools(
        viewport_size: UVec2,
        settings_mutations: Arc<SettingsMutationCoordinator>,
        tool_scheduler: ToolSchedulerService,
        viewport_id: ViewInstanceId,
    ) -> Self {
        let scene_tool_resources = ToolResourceSet::pair(
            ToolResourceKey::viewport_input(viewport_id.clone()),
            ToolResourceKey::scene_mode_slot(viewport_id),
        );
        Self {
            state: SceneViewportState::new(viewport_size),
            handles: HandleToolRegistry::default(),
            interaction_extract: ViewportInteractionExtractCache::default(),
            pointer_bridge: ViewportOverlayPointerRouter::new(),
            settings_mutations,
            overlay_providers: Default::default(),
            runtime_overlay_ui_cache: Default::default(),
            tool_scheduler,
            scene_tool_identity: SceneToolIdentity::Pending,
            scene_tool_resources,
            scene_tool_lease: None,
        }
    }
}
