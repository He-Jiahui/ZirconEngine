use std::sync::{Arc, Mutex};

use crate::scene::viewport::RenderFramework;

use super::slint_viewport_controller::SlintViewportController;
use super::viewport_state::ViewportState;

impl SlintViewportController {
    pub(super) fn new_with_framework(render_framework: Arc<dyn RenderFramework>) -> Self {
        Self {
            shared: Arc::new(Mutex::new(ViewportState {
                render_framework,
                viewport: None,
                latest_generation: None,
                latest_image: None,
                last_error: None,
                last_world_space_ui_surfaces: Vec::new(),
                world_space_ui_pointer_capture: None,
            })),
        }
    }
}
