use crate::scene::viewport::{RenderFrameExtract, RenderFrameworkError};
use zircon_runtime::core::math::UVec2;

use super::slint_viewport_controller::SlintViewportController;

impl SlintViewportController {
    pub(crate) fn submit_extract(
        &self,
        mut extract: RenderFrameExtract,
        size: UVec2,
    ) -> Result<(), RenderFrameworkError> {
        let mut shared = self.shared.lock().unwrap();
        let viewport = shared.ensure_viewport(size)?;
        extract.apply_viewport_size(size);
        shared
            .render_framework
            .submit_frame_extract(viewport, extract)?;
        shared.last_error = None;
        Ok(())
    }
}
