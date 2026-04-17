use zircon_core::{CoreError, CoreHandle};
use zircon_render_server::resolve_render_server;

use super::slint_viewport_controller::SlintViewportController;

impl SlintViewportController {
    pub(crate) fn new(core: CoreHandle) -> Result<Self, CoreError> {
        let render_server = resolve_render_server(&core)?;
        Ok(Self::new_with_server(render_server))
    }
}
