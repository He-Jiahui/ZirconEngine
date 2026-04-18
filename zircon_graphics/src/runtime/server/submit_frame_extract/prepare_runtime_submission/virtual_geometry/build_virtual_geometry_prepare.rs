use crate::runtime::VirtualGeometryRuntimeState;
use crate::types::VirtualGeometryPrepareFrame;
use crate::VisibilityVirtualGeometryCluster;

pub(in crate::runtime::server::submit_frame_extract::prepare_runtime_submission) fn build_virtual_geometry_prepare(
    runtime: Option<&VirtualGeometryRuntimeState>,
    visible_clusters: &[VisibilityVirtualGeometryCluster],
    visibility_draw_segments: &[crate::VisibilityVirtualGeometryDrawSegment],
) -> Option<VirtualGeometryPrepareFrame> {
    runtime.map(|runtime| {
        runtime.build_prepare_frame_with_segments(visible_clusters, visibility_draw_segments)
    })
}
