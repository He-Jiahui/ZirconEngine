use std::sync::Arc;

use crate::scene::viewport::RenderFramework;
use crate::ui::slint_host::host_contract::WorldSpaceUiSurfaceSubmission;
use slint::Image;

use super::active_viewport::ActiveViewport;

pub(super) struct ViewportState {
    pub(super) render_framework: Arc<dyn RenderFramework>,
    pub(super) viewport: Option<ActiveViewport>,
    pub(super) latest_generation: Option<u64>,
    pub(super) latest_image: Option<Image>,
    pub(super) last_error: Option<String>,
    #[allow(dead_code)]
    pub(super) last_world_space_ui_surfaces: Vec<WorldSpaceUiSurfaceSubmission>,
    pub(super) world_space_ui_pointer_capture: Option<WorldSpaceUiSurfaceSubmission>,
}
