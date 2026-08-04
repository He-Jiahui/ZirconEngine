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
        let _operation = self.lock_viewport_lifecycle();
        let Some((viewport, render_framework)) = self.ensure_viewport(size)? else {
            return Ok(false);
        };
        let ui = {
            let shared = self.lock_shared();
            merge_ui_with_world_space_submissions(ui, &shared.last_world_space_ui_surfaces)
        };
        extract.apply_viewport_size(size);
        apply_editor_viewport_render_defaults(&mut extract);
        render_framework.submit_frame_extract_with_ui(viewport, extract, ui)?;
        let mut shared = self.lock_shared();
        if shared
            .viewport
            .is_some_and(|active| active.handle == viewport)
        {
            shared.last_error = None;
        }
        Ok(true)
    }

    pub(crate) fn visible_spatial_snapshot(
        &self,
    ) -> Result<Option<RenderVisibleSpatialQuerySnapshot>, RenderFrameworkError> {
        let _operation = self.lock_viewport_lifecycle();
        let (viewport, render_framework) = {
            let shared = self.lock_shared();
            let Some(viewport) = shared.viewport else {
                return Ok(None);
            };
            let Some(render_framework) = shared.resolve_stored_render_framework()? else {
                return Ok(None);
            };
            (viewport.handle, render_framework)
        };
        render_framework.query_visible_spatial_snapshot(viewport)
    }

    #[cfg(test)]
    pub(crate) fn submit_extract(
        &self,
        mut extract: RenderFrameExtract,
        size: UVec2,
    ) -> Result<bool, RenderFrameworkError> {
        zircon_runtime::profile_scope!("editor", "viewport", "submit_extract");
        let _operation = self.lock_viewport_lifecycle();
        let Some((viewport, render_framework)) = self.ensure_viewport(size)? else {
            return Ok(false);
        };
        extract.apply_viewport_size(size);
        apply_editor_viewport_render_defaults(&mut extract);
        render_framework.submit_frame_extract(viewport, extract)?;
        let mut shared = self.lock_shared();
        if shared
            .viewport
            .is_some_and(|active| active.handle == viewport)
        {
            shared.last_error = None;
        }
        Ok(true)
    }
}
