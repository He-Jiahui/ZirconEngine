use std::sync::{Arc, Mutex};

use super::render_framework_access::ViewportRenderFrameworkAccess;
use super::retained_viewport_controller::RetainedViewportController;
use super::viewport_state::ViewportState;

impl RetainedViewportController {
    pub(in crate::ui::retained_host) fn new(
        render_framework_access: ViewportRenderFrameworkAccess,
    ) -> Self {
        zircon_runtime::profile_scope!("editor", "viewport", "controller_new");
        {
            zircon_runtime::profile_scope!("editor", "viewport", "controller_build_lazy_state");
            Self {
                shared: Arc::new(Mutex::new(ViewportState::lazy(render_framework_access))),
                viewport_lifecycle: Arc::new(Mutex::new(())),
            }
        }
    }
}
