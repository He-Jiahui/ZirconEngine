use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
};
use crate::graphics::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalRecord,
};

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryNodeAndClusterCullPassStoreParts
{
    pub(in crate::graphics::scene::scene_renderer::core) source:
        RenderVirtualGeometryNodeAndClusterCullSource,
    pub(in crate::graphics::scene::scene_renderer::core) record_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) global_state:
        Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub(in crate::graphics::scene::scene_renderer::core) buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) dispatch_setup:
        Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub(in crate::graphics::scene::scene_renderer::core) launch_worklist:
        Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub(in crate::graphics::scene::scene_renderer::core) dispatch_setup_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) launch_worklist_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) instance_seed_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) instance_seeds:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub(in crate::graphics::scene::scene_renderer::core) instance_seed_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) instance_work_item_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) instance_work_items:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub(in crate::graphics::scene::scene_renderer::core) instance_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) cluster_work_item_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) cluster_work_items:
        Vec<VirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub(in crate::graphics::scene::scene_renderer::core) cluster_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) hierarchy_child_ids: Vec<u32>,
    pub(in crate::graphics::scene::scene_renderer::core) hierarchy_child_id_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) child_work_item_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) child_work_items:
        Vec<VirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub(in crate::graphics::scene::scene_renderer::core) child_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) traversal_record_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) traversal_records:
        Vec<VirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub(in crate::graphics::scene::scene_renderer::core) traversal_record_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) page_request_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) page_request_ids: Vec<u32>,
    pub(in crate::graphics::scene::scene_renderer::core) page_request_buffer:
        Option<Arc<wgpu::Buffer>>,
}
