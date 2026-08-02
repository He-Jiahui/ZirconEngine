use crate::scene::viewport::{
    RenderFrameExtract, RenderFrameworkError, RenderVisibleSpatialQuerySnapshot,
};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::editor_viewport_render_defaults::apply_editor_viewport_render_defaults;
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
        apply_editor_viewport_render_defaults(&mut extract);
        let ui = merge_ui_with_world_space_submissions(ui, &shared.last_world_space_ui_surfaces);
        let render_framework = shared.render_framework()?;
        render_framework.submit_frame_extract_with_ui(viewport, extract, ui)?;
        shared.last_error = None;
        Ok(true)
    }

    pub(crate) fn visible_spatial_snapshot(
        &self,
    ) -> Result<Option<RenderVisibleSpatialQuerySnapshot>, RenderFrameworkError> {
        let shared = self.lock_shared();
        let Some(viewport) = shared.viewport else {
            return Ok(None);
        };
        let Some(render_framework) = shared.resolve_stored_render_framework()? else {
            return Ok(None);
        };
        render_framework.query_visible_spatial_snapshot(viewport.handle)
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
        apply_editor_viewport_render_defaults(&mut extract);
        let render_framework = shared.render_framework()?;
        render_framework.submit_frame_extract(viewport, extract)?;
        shared.last_error = None;
        Ok(true)
    }
}
