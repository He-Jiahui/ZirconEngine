use super::viewport_render_frame::ViewportRenderFrame;
use super::virtual_geometry_prepare::VirtualGeometryPrepareFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_virtual_geometry_prepare(
        mut self,
        prepare: Option<VirtualGeometryPrepareFrame>,
    ) -> Self {
        self.virtual_geometry_prepare = prepare;
        self
    }
}
