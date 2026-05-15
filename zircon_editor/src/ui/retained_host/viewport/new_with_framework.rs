use std::sync::{Arc, Mutex};

use crate::scene::viewport::RenderFramework;

use super::retained_viewport_controller::RetainedViewportController;
use super::viewport_state::ViewportState;

impl RetainedViewportController {
    pub(super) fn new_with_framework(render_framework: Arc<dyn RenderFramework>) -> Self {
        Self {
            shared: Arc::new(Mutex::new(ViewportState::with_render_framework(
                render_framework,
            ))),
        }
    }
}
