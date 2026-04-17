use crate::runtime::VirtualGeometryRuntimeState;
use crate::types::VirtualGeometryPrepareFrame;
use crate::VisibilityVirtualGeometryCluster;

pub(super) fn build_virtual_geometry_prepare(
    runtime: Option<&VirtualGeometryRuntimeState>,
    visible_clusters: &[VisibilityVirtualGeometryCluster],
) -> Option<VirtualGeometryPrepareFrame> {
    runtime.map(|runtime| runtime.build_prepare_frame(visible_clusters))
}
