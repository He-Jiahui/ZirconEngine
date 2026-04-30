use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
};
use crate::graphics::scene::scene_renderer::core::scene_renderer::VirtualGeometryCullOutputUpdate;

#[derive(Default)]
pub(super) struct VirtualGeometryCullOutputs {
    cluster_selection_input_source: RenderVirtualGeometryClusterSelectionInputSource,
    cull_input_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_source: RenderVirtualGeometryNodeAndClusterCullSource,
    node_and_cluster_cull_record_count: u32,
    node_and_cluster_cull_global_state:
        Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    node_and_cluster_cull_dispatch_group_count: [u32; 3],
    node_and_cluster_cull_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_dispatch_setup_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_launch_worklist_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_instance_seed_count: u32,
    node_and_cluster_cull_instance_seed_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_instance_work_item_count: u32,
    node_and_cluster_cull_instance_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_cluster_work_item_count: u32,
    node_and_cluster_cull_cluster_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_hierarchy_child_id_count: u32,
    node_and_cluster_cull_hierarchy_child_id_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_child_work_item_count: u32,
    node_and_cluster_cull_child_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_traversal_record_count: u32,
    node_and_cluster_cull_traversal_record_buffer: Option<Arc<wgpu::Buffer>>,
    node_and_cluster_cull_page_request_count: u32,
    node_and_cluster_cull_page_request_ids: Vec<u32>,
    node_and_cluster_cull_page_request_buffer: Option<Arc<wgpu::Buffer>>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl VirtualGeometryCullOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn cluster_selection_input_source(
        &self,
    ) -> RenderVirtualGeometryClusterSelectionInputSource {
        self.cluster_selection_input_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn cull_input_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.cull_input_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_source(
        &self,
    ) -> RenderVirtualGeometryNodeAndClusterCullSource {
        self.node_and_cluster_cull_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_record_count(
        &self,
    ) -> u32 {
        self.node_and_cluster_cull_record_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_global_state(
        &self,
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot> {
        self.node_and_cluster_cull_global_state.clone()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_dispatch_group_count(
        &self,
    ) -> [u32; 3] {
        self.node_and_cluster_cull_dispatch_group_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_dispatch_setup_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_dispatch_setup_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_launch_worklist_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_launch_worklist_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_instance_seed_count(
        &self,
    ) -> u32 {
        self.node_and_cluster_cull_instance_seed_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_instance_seed_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_instance_seed_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_instance_work_item_count(
        &self,
    ) -> u32 {
        self.node_and_cluster_cull_instance_work_item_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_instance_work_item_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_instance_work_item_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_cluster_work_item_count(
        &self,
    ) -> u32 {
        self.node_and_cluster_cull_cluster_work_item_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_cluster_work_item_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_cluster_work_item_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_hierarchy_child_id_count(
        &self,
    ) -> u32 {
        self.node_and_cluster_cull_hierarchy_child_id_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_hierarchy_child_id_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_hierarchy_child_id_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_child_work_item_count(
        &self,
    ) -> u32 {
        self.node_and_cluster_cull_child_work_item_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_child_work_item_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_child_work_item_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_traversal_record_count(
        &self,
    ) -> u32 {
        self.node_and_cluster_cull_traversal_record_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_traversal_record_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_traversal_record_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_page_request_ids(
        &self,
    ) -> &[u32] {
        &self.node_and_cluster_cull_page_request_ids
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_page_request_count(
        &self,
    ) -> u32 {
        self.node_and_cluster_cull_page_request_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn node_and_cluster_cull_page_request_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.node_and_cluster_cull_page_request_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store(
        &mut self,
        update: VirtualGeometryCullOutputUpdate,
    ) {
        self.cluster_selection_input_source = update.cluster_selection_input_source;
        self.cull_input_buffer = update.cull_input_buffer;
        self.node_and_cluster_cull_source = update.node_and_cluster_cull_source;
        self.node_and_cluster_cull_record_count = update.node_and_cluster_cull_record_count;
        self.node_and_cluster_cull_global_state = update.node_and_cluster_cull_global_state;
        self.node_and_cluster_cull_dispatch_group_count =
            update.node_and_cluster_cull_dispatch_group_count;
        self.node_and_cluster_cull_buffer = update.node_and_cluster_cull_buffer;
        self.node_and_cluster_cull_dispatch_setup_buffer =
            update.node_and_cluster_cull_dispatch_setup_buffer;
        self.node_and_cluster_cull_launch_worklist_buffer =
            update.node_and_cluster_cull_launch_worklist_buffer;
        self.node_and_cluster_cull_instance_seed_count =
            update.node_and_cluster_cull_instance_seed_count;
        self.node_and_cluster_cull_instance_seed_buffer =
            update.node_and_cluster_cull_instance_seed_buffer;
        self.node_and_cluster_cull_instance_work_item_count =
            update.node_and_cluster_cull_instance_work_item_count;
        self.node_and_cluster_cull_instance_work_item_buffer =
            update.node_and_cluster_cull_instance_work_item_buffer;
        self.node_and_cluster_cull_cluster_work_item_count =
            update.node_and_cluster_cull_cluster_work_item_count;
        self.node_and_cluster_cull_cluster_work_item_buffer =
            update.node_and_cluster_cull_cluster_work_item_buffer;
        self.node_and_cluster_cull_hierarchy_child_id_count =
            update.node_and_cluster_cull_hierarchy_child_id_count;
        self.node_and_cluster_cull_hierarchy_child_id_buffer =
            update.node_and_cluster_cull_hierarchy_child_id_buffer;
        self.node_and_cluster_cull_child_work_item_count =
            update.node_and_cluster_cull_child_work_item_count;
        self.node_and_cluster_cull_child_work_item_buffer =
            update.node_and_cluster_cull_child_work_item_buffer;
        self.node_and_cluster_cull_traversal_record_count =
            update.node_and_cluster_cull_traversal_record_count;
        self.node_and_cluster_cull_traversal_record_buffer =
            update.node_and_cluster_cull_traversal_record_buffer;
        self.node_and_cluster_cull_page_request_count =
            update.node_and_cluster_cull_page_request_count;
        self.node_and_cluster_cull_page_request_ids = update.node_and_cluster_cull_page_request_ids;
        self.node_and_cluster_cull_page_request_buffer =
            update.node_and_cluster_cull_page_request_buffer;
    }
}
