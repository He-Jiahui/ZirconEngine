use std::cell::RefCell;
use std::sync::Arc;

use crate::core::context::{ToolSchedulerService, ToolSchedulerServiceError};
use crate::core::settings::SettingsMutationCoordinator;
use crate::core::tools::{ToolInstanceId, ToolLeaseHandle, ToolResourceSet};
use crate::scene::viewport::handles::HandleToolRegistry;
use crate::scene::viewport::pointer::ViewportOverlayPointerRouter;
use crate::scene::viewport::ViewportInteractionExtractCache;

use super::scene_viewport_controller_build_runtime_overlay_ui::RuntimeOverlayUiExtractCache;
use super::scene_viewport_controller_overlay_providers::ViewportOverlayProviderRegistry;
use super::scene_viewport_state::SceneViewportState;

pub(crate) struct SceneViewportController {
    pub(in crate::scene::viewport::controller) state: SceneViewportState,
    pub(in crate::scene::viewport::controller) handles: HandleToolRegistry,
    pub(in crate::scene::viewport::controller) interaction_extract: ViewportInteractionExtractCache,
    pub(in crate::scene::viewport::controller) pointer_bridge: ViewportOverlayPointerRouter,
    pub(in crate::scene::viewport::controller) settings_mutations: Arc<SettingsMutationCoordinator>,
    pub(in crate::scene::viewport::controller) overlay_providers: ViewportOverlayProviderRegistry,
    pub(in crate::scene::viewport::controller) runtime_overlay_ui_cache:
        RefCell<RuntimeOverlayUiExtractCache>,
    pub(in crate::scene::viewport::controller) tool_scheduler: ToolSchedulerService,
    pub(in crate::scene::viewport::controller) scene_tool_identity: SceneToolIdentity,
    pub(in crate::scene::viewport::controller) scene_tool_resources: ToolResourceSet,
    pub(in crate::scene::viewport::controller) scene_tool_lease: Option<ToolLeaseHandle>,
}

#[derive(Debug)]
pub(in crate::scene::viewport::controller) enum SceneToolIdentity {
    Pending,
    Allocated(ToolInstanceId),
    Failed(ToolSchedulerServiceError),
}

impl SceneToolIdentity {
    pub(super) fn allocated(&self) -> Option<&ToolInstanceId> {
        match self {
            Self::Allocated(tool_id) => Some(tool_id),
            Self::Pending | Self::Failed(_) => None,
        }
    }
}

impl Drop for SceneViewportController {
    fn drop(&mut self) {
        if let Some(lease) = self.scene_tool_lease.take() {
            let _ = self.tool_scheduler.release(lease.id());
        }
    }
}
