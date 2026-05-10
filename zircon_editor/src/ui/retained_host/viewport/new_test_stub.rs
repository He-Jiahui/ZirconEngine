use std::sync::Arc;

use super::retained_viewport_controller::RetainedViewportController;
use super::test_render_framework::TestRenderFramework;

impl RetainedViewportController {
    pub(crate) fn new_test_stub() -> Self {
        Self::new_with_framework(Arc::new(TestRenderFramework))
    }
}
