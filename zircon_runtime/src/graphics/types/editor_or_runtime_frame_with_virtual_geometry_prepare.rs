use super::editor_or_runtime_frame::EditorOrRuntimeFrame;
use super::virtual_geometry_prepare::VirtualGeometryPrepareFrame;

impl EditorOrRuntimeFrame {
    pub(crate) fn with_virtual_geometry_prepare(
        mut self,
        prepare: Option<VirtualGeometryPrepareFrame>,
    ) -> Self {
        self.virtual_geometry_prepare = prepare;
        self
    }
}
