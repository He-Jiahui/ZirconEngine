use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource,
};

use super::virtual_geometry_hardware_rasterization_pass::VirtualGeometryHardwareRasterizationPassOutput;
use super::virtual_geometry_node_and_cluster_cull_pass::VirtualGeometryNodeAndClusterCullPassOutput;
use super::virtual_geometry_visbuffer64_pass::VirtualGeometryVisBuffer64PassOutput;

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryIndirectStatsStoreParts {
    pub(in crate::graphics::scene::scene_renderer::core) draw_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) buffer_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_page_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_resident_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_pending_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_missing_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_repeated_draw_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_indirect_offsets: Vec<u64>,
    pub(in crate::graphics::scene::scene_renderer::core) execution_segments:
        Vec<RenderVirtualGeometryExecutionSegment>,
    pub(in crate::graphics::scene::scene_renderer::core) executed_selected_clusters:
        Vec<RenderVirtualGeometrySelectedCluster>,
    pub(in crate::graphics::scene::scene_renderer::core) executed_selected_cluster_source:
        RenderVirtualGeometrySelectedClusterSource,
    pub(in crate::graphics::scene::scene_renderer::core) executed_selected_cluster_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) executed_selected_cluster_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_pass:
        VirtualGeometryNodeAndClusterCullPassOutput,
    pub(in crate::graphics::scene::scene_renderer::core) hardware_rasterization_pass:
        VirtualGeometryHardwareRasterizationPassOutput,
    pub(in crate::graphics::scene::scene_renderer::core) visbuffer64_pass:
        VirtualGeometryVisBuffer64PassOutput,
    pub(in crate::graphics::scene::scene_renderer::core) draw_submission_order:
        Vec<(Option<u32>, u64, u32)>,
    pub(in crate::graphics::scene::scene_renderer::core) draw_submission_records:
        Vec<(u64, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) draw_submission_token_records:
        Vec<(u64, u32, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) args_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) authority_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) segment_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) execution_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) execution_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) execution_authority_buffer:
        Option<Arc<wgpu::Buffer>>,
}
