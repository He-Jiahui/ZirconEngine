use super::viewport_render_frame::{
    ViewportRenderFrame, ViewportVirtualGeometryClusterSelectionSource,
};
use super::virtual_geometry_cluster_selection::VirtualGeometryClusterSelection;

impl ViewportRenderFrame {
    pub(crate) fn with_virtual_geometry_cluster_selections(
        mut self,
        selections: Option<Vec<VirtualGeometryClusterSelection>>,
    ) -> Self {
        self.virtual_geometry_cluster_selections = selections;
        self.virtual_geometry_cluster_selections_source = self
            .virtual_geometry_cluster_selections
            .as_ref()
            .map(|_| ViewportVirtualGeometryClusterSelectionSource::ExplicitFrameOwned);
        self
    }
}
