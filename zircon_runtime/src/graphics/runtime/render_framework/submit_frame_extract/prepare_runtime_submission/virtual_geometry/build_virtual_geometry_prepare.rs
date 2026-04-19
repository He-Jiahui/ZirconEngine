use crate::graphics::runtime::VirtualGeometryRuntimeState;
use crate::graphics::types::VirtualGeometryPrepareFrame;
use crate::VisibilityVirtualGeometryCluster;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract::prepare_runtime_submission) fn build_virtual_geometry_prepare(
    runtime: Option<&VirtualGeometryRuntimeState>,
    visible_clusters: &[VisibilityVirtualGeometryCluster],
    visibility_draw_segments: &[crate::graphics::VisibilityVirtualGeometryDrawSegment],
) -> Option<VirtualGeometryPrepareFrame> {
    runtime.map(|runtime| {
        runtime.build_prepare_frame_with_segments(visible_clusters, visibility_draw_segments)
    })
}
