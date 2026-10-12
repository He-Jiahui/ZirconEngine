use crate::core::framework::render::RenderVirtualGeometryDebugSnapshot;
use std::sync::Arc;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_virtual_geometry_debug_snapshot(
        mut self,
        snapshot: Option<Arc<RenderVirtualGeometryDebugSnapshot>>,
    ) -> Self {
        self.virtual_geometry_debug_snapshot = snapshot;
        self
    }
}
