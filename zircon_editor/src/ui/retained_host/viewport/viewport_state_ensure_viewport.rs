use crate::scene::viewport::{
    RenderFrameworkError, RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_runtime_interface::math::UVec2;

use super::active_viewport::ActiveViewport;
use super::viewport_state::ViewportState;

impl ViewportState {
    pub(super) fn ensure_viewport(
        &mut self,
        size: UVec2,
    ) -> Result<Option<RenderViewportHandle>, RenderFrameworkError> {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        let Some(render_framework) = self.poll_or_start_render_framework()? else {
            return Ok(None);
        };
        if let Some(viewport) = self.viewport {
            if viewport.size == size {
                return Ok(Some(viewport.handle));
            }
            render_framework.destroy_viewport(viewport.handle)?;
            self.viewport = None;
            self.latest_generation = None;
            self.latest_image = None;
        }

        let descriptor = RenderViewportDescriptor::new(size).with_label("editor.viewport");
        let handle = render_framework.create_viewport(descriptor)?;
        self.viewport = Some(ActiveViewport { handle, size });
        Ok(Some(handle))
    }
}
