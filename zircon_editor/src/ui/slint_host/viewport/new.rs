use zircon_runtime::core::manager::resolve_render_framework;
use zircon_runtime::core::{CoreError, CoreHandle};

use super::slint_viewport_controller::SlintViewportController;

impl SlintViewportController {
    pub(crate) fn new(core: CoreHandle) -> Result<Self, CoreError> {
        let render_framework = resolve_render_framework(&core)?;
        Ok(Self::new_with_framework(render_framework))
    }
}
