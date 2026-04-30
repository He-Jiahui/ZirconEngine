use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
};

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryCullOutputUpdate {
    pub(in crate::graphics::scene::scene_renderer::core) cluster_selection_input_source:
        RenderVirtualGeometryClusterSelectionInputSource,
    pub(in crate::graphics::scene::scene_renderer::core) cull_input_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_source:
        RenderVirtualGeometryNodeAndClusterCullSource,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_record_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_global_state:
        Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_dispatch_group_count:
        [u32; 3],
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_dispatch_setup_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_launch_worklist_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_instance_seed_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_instance_seed_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_instance_work_item_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_instance_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_cluster_work_item_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_cluster_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_hierarchy_child_id_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_hierarchy_child_id_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_child_work_item_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_child_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_traversal_record_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_traversal_record_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_page_request_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_page_request_ids:
        Vec<u32>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_page_request_buffer:
        Option<Arc<wgpu::Buffer>>,
}
