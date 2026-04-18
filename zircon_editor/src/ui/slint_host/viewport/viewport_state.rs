use std::sync::Arc;

use slint::Image;
use zircon_framework::render::RenderFramework;

use super::active_viewport::ActiveViewport;

pub(super) struct ViewportState {
    pub(super) render_framework: Arc<dyn RenderFramework>,
    pub(super) viewport: Option<ActiveViewport>,
    pub(super) latest_generation: Option<u64>,
    pub(super) latest_image: Option<Image>,
    pub(super) last_error: Option<String>,
}
