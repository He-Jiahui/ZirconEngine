use std::sync::Arc;

use super::slint_viewport_controller::SlintViewportController;
use super::test_render_server::TestRenderServer;

impl SlintViewportController {
    pub(crate) fn new_test_stub() -> Self {
        Self::new_with_server(Arc::new(TestRenderServer))
    }
}
