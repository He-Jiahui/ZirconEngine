use zircon_math::UVec2;
use zircon_render_server::{RenderServerError, RenderViewportDescriptor, RenderViewportHandle};

use super::active_viewport::ActiveViewport;
use super::viewport_state::ViewportState;

impl ViewportState {
    pub(super) fn ensure_viewport(
        &mut self,
        size: UVec2,
    ) -> Result<RenderViewportHandle, RenderServerError> {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        if let Some(viewport) = self.viewport {
            if viewport.size == size {
                return Ok(viewport.handle);
            }
            self.render_server.destroy_viewport(viewport.handle)?;
            self.viewport = None;
            self.latest_generation = None;
            self.latest_image = None;
        }

        let descriptor = RenderViewportDescriptor::new(size).with_label("editor.viewport");
        let handle = self.render_server.create_viewport(descriptor)?;
        self.viewport = Some(ActiveViewport { handle, size });
        Ok(handle)
    }
}
