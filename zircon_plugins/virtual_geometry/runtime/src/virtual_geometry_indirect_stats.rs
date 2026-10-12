use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource,
};

use super::virtual_geometry_hardware_rasterization_pass::VirtualGeometryHardwareRasterizationPassOutput;
use super::virtual_geometry_node_and_cluster_cull_pass::VirtualGeometryNodeAndClusterCullPassOutput;
use super::virtual_geometry_visbuffer64_pass::VirtualGeometryVisBuffer64PassOutput;

pub(in crate::virtual_geometry::renderer) struct VirtualGeometryIndirectStatsStoreParts {
    pub(in crate::virtual_geometry::renderer) draw_count: u32,
    pub(in crate::virtual_geometry::renderer) buffer_count: u32,
    pub(in crate::virtual_geometry::renderer) segment_count: u32,
    pub(in crate::virtual_geometry::renderer) execution_segment_count: u32,
    pub(in crate::virtual_geometry::renderer) execution_page_count: u32,
    pub(in crate::virtual_geometry::renderer) execution_resident_segment_count: u32,
    pub(in crate::virtual_geometry::renderer) execution_pending_segment_count: u32,
    pub(in crate::virtual_geometry::renderer) execution_missing_segment_count: u32,
    pub(in crate::virtual_geometry::renderer) execution_repeated_draw_count: u32,
    pub(in crate::virtual_geometry::renderer) execution_indirect_offsets: Vec<u64>,
    pub(in crate::virtual_geometry::renderer) execution_segments:
        Vec<RenderVirtualGeometryExecutionSegment>,
    pub(in crate::virtual_geometry::renderer) executed_selected_clusters:
        Vec<RenderVirtualGeometrySelectedCluster>,
    pub(in crate::virtual_geometry::renderer) executed_selected_cluster_source:
        RenderVirtualGeometrySelectedClusterSource,
    pub(in crate::virtual_geometry::renderer) executed_selected_cluster_count: u32,
    pub(in crate::virtual_geometry::renderer) executed_selected_cluster_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) node_and_cluster_cull_pass:
        VirtualGeometryNodeAndClusterCullPassOutput,
    pub(in crate::virtual_geometry::renderer) hardware_rasterization_pass:
        VirtualGeometryHardwareRasterizationPassOutput,
    pub(in crate::virtual_geometry::renderer) visbuffer64_pass:
        VirtualGeometryVisBuffer64PassOutput,
    pub(in crate::virtual_geometry::renderer) draw_submission_order: Vec<(Option<u32>, u64, u32)>,
    pub(in crate::virtual_geometry::renderer) draw_submission_records: Vec<(u64, u32, u32, usize)>,
    pub(in crate::virtual_geometry::renderer) draw_submission_token_records:
        Vec<(u64, u32, u32, u32, usize)>,
    pub(in crate::virtual_geometry::renderer) args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) args_count: u32,
    pub(in crate::virtual_geometry::renderer) submission_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) authority_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) segment_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) execution_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) execution_args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) execution_authority_buffer: Option<Arc<wgpu::Buffer>>,
}
