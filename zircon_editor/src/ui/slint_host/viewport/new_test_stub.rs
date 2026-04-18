use std::sync::Arc;

use super::slint_viewport_controller::SlintViewportController;
use super::test_render_framework::TestRenderFramework;

impl SlintViewportController {
    pub(crate) fn new_test_stub() -> Self {
        Self::new_with_framework(Arc::new(TestRenderFramework))
    }
}
