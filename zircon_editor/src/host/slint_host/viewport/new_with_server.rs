use std::sync::{Arc, Mutex};

use zircon_render_server::RenderServer;

use super::slint_viewport_controller::SlintViewportController;
use super::viewport_state::ViewportState;

impl SlintViewportController {
    pub(super) fn new_with_server(render_server: Arc<dyn RenderServer>) -> Self {
        Self {
            shared: Arc::new(Mutex::new(ViewportState {
                render_server,
                viewport: None,
                latest_generation: None,
                latest_image: None,
                last_error: None,
            })),
        }
    }
}
