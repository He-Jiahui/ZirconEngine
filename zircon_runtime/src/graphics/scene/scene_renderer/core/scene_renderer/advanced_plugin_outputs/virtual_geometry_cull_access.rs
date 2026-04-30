use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
};

use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;

#[cfg_attr(not(test), allow(dead_code))]
impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_cluster_selection_input_source(
        &self,
    ) -> RenderVirtualGeometryClusterSelectionInputSource {
        self.virtual_geometry_cull()
            .cluster_selection_input_source()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_cull_input_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull().cull_input_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_source(
        &self,
    ) -> RenderVirtualGeometryNodeAndClusterCullSource {
        self.virtual_geometry_cull().node_and_cluster_cull_source()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_record_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_record_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_dispatch_group_count(
        &self,
    ) -> [u32; 3] {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_dispatch_group_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull().node_and_cluster_cull_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_dispatch_setup_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_dispatch_setup_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_launch_worklist_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_launch_worklist_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_instance_seed_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_instance_seed_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_instance_seed_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_instance_seed_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_instance_work_item_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_instance_work_item_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_instance_work_item_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_instance_work_item_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_cluster_work_item_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_cluster_work_item_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_cluster_work_item_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_cluster_work_item_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_hierarchy_child_id_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_hierarchy_child_id_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_hierarchy_child_id_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_child_work_item_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_child_work_item_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_child_work_item_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_child_work_item_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_traversal_record_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_traversal_record_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_traversal_record_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_traversal_record_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_page_request_ids(
        &self,
    ) -> &[u32] {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_page_request_ids()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_page_request_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_page_request_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_page_request_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_cull()
            .node_and_cluster_cull_page_request_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn previous_virtual_geometry_node_and_cluster_cull_global_state(
        &self,
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot> {
        self.virtual_geometry_render_path()
            .debug_snapshot_node_and_cluster_cull_global_state()
            .or_else(|| {
                self.virtual_geometry_cull()
                    .node_and_cluster_cull_global_state()
            })
    }
}
