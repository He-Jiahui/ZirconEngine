use std::sync::{Arc, Mutex};

use zircon_runtime::core::{CoreError, CoreHandle};

use super::retained_viewport_controller::RetainedViewportController;
use super::viewport_state::ViewportState;

impl RetainedViewportController {
    pub(crate) fn new(core: CoreHandle) -> Result<Self, CoreError> {
        zircon_runtime::profile_scope!("editor", "viewport", "controller_new");
        Ok({
            zircon_runtime::profile_scope!("editor", "viewport", "controller_build_lazy_state");
            Self {
                shared: Arc::new(Mutex::new(ViewportState::lazy(core))),
            }
        })
    }
}
