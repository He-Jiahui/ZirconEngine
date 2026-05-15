use crate::scene::viewport::{RenderFrameExtract, RenderFrameworkError};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::retained_viewport_controller::RetainedViewportController;
use super::world_space_ui::merge_ui_with_world_space_submissions;

impl RetainedViewportController {
    pub(crate) fn submit_extract_with_ui(
        &self,
        mut extract: RenderFrameExtract,
        ui: Option<UiRenderExtract>,
        size: UVec2,
    ) -> Result<bool, RenderFrameworkError> {
        zircon_runtime::profile_scope!("editor", "viewport", "submit_extract_with_ui");
        let mut shared = self.lock_shared();
        let Some(viewport) = shared.ensure_viewport(size)? else {
            return Ok(false);
        };
        extract.apply_viewport_size(size);
        let ui = merge_ui_with_world_space_submissions(ui, &shared.last_world_space_ui_surfaces);
        let render_framework = shared.render_framework()?;
        render_framework.submit_frame_extract_with_ui(viewport, extract, ui)?;
        shared.last_error = None;
        Ok(true)
    }

    #[cfg(test)]
    pub(crate) fn submit_extract(
        &self,
        mut extract: RenderFrameExtract,
        size: UVec2,
    ) -> Result<bool, RenderFrameworkError> {
        zircon_runtime::profile_scope!("editor", "viewport", "submit_extract");
        let mut shared = self.lock_shared();
        let Some(viewport) = shared.ensure_viewport(size)? else {
            return Ok(false);
        };
        extract.apply_viewport_size(size);
        let render_framework = shared.render_framework()?;
        render_framework.submit_frame_extract(viewport, extract)?;
        shared.last_error = None;
        Ok(true)
    }
}
