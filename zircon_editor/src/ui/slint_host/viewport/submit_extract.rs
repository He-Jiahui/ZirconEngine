use crate::scene::viewport::{RenderFrameExtract, RenderFrameworkError};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::ui::surface::UiRenderExtract;

use super::slint_viewport_controller::SlintViewportController;

impl SlintViewportController {
    pub(crate) fn submit_extract_with_ui(
        &self,
        mut extract: RenderFrameExtract,
        ui: Option<UiRenderExtract>,
        size: UVec2,
    ) -> Result<(), RenderFrameworkError> {
        let mut shared = self.shared.lock().unwrap();
        let viewport = shared.ensure_viewport(size)?;
        extract.apply_viewport_size(size);
        shared
            .render_framework
            .submit_frame_extract_with_ui(viewport, extract, ui)?;
        shared.last_error = None;
        Ok(())
    }

    #[cfg(test)]
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
