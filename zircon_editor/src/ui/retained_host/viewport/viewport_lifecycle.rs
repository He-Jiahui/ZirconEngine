use std::sync::Arc;

use crate::scene::viewport::{
    RenderFramework, RenderFrameworkError, RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_runtime_interface::math::UVec2;

use super::active_viewport::ActiveViewport;
use super::editor_viewport_render_defaults::editor_viewport_quality_profile;
use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(super) fn ensure_viewport(
        &self,
        size: UVec2,
    ) -> Result<Option<(RenderViewportHandle, Arc<dyn RenderFramework>)>, RenderFrameworkError>
    {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        let (render_framework, previous) = {
            let mut shared = self.lock_shared();
            let Some(render_framework) = shared.poll_or_start_render_framework()? else {
                return Ok(None);
            };
            if let Some(viewport) = shared.viewport {
                if viewport.size == size {
                    return Ok(Some((viewport.handle, render_framework)));
                }
            }
            (render_framework, shared.viewport)
        };

        if let Some(viewport) = previous {
            self.clear_viewport_if_current(viewport.handle);
            if let Err(error) = render_framework.destroy_viewport(viewport.handle) {
                self.restore_viewport_if_empty(viewport);
                return Err(error);
            }
        }

        let descriptor = RenderViewportDescriptor::new(size).with_label("editor.viewport");
        let handle = render_framework.create_viewport(descriptor)?;
        if let Err(error) =
            render_framework.set_quality_profile(handle, editor_viewport_quality_profile())
        {
            let _ = render_framework.destroy_viewport(handle);
            return Err(error);
        }

        let mut shared = self.lock_shared();
        shared.viewport = Some(ActiveViewport { handle, size });
        shared.latest_generation = None;
        Ok(Some((handle, render_framework)))
    }

    fn clear_viewport_if_current(&self, expected: RenderViewportHandle) {
        let mut shared = self.lock_shared();
        if shared
            .viewport
            .is_some_and(|active| active.handle == expected)
        {
            shared.viewport = None;
            shared.latest_generation = None;
        }
    }

    fn restore_viewport_if_empty(&self, viewport: ActiveViewport) {
        let mut shared = self.lock_shared();
        if shared.viewport.is_none() {
            shared.viewport = Some(viewport);
        }
    }
}
