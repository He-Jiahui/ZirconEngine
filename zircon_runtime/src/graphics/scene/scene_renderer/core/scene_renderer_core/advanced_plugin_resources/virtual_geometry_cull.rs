use std::sync::Arc;

use crate::core::framework::render::RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot;

use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn create_virtual_geometry_node_and_cluster_cull_instance_work_item_buffer(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        launch_worklist_buffer: Option<&Arc<wgpu::Buffer>>,
        dispatch_setup: RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
        instance_work_item_count: u32,
    ) -> Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry().and_then(|virtual_geometry| {
            virtual_geometry.create_node_and_cluster_cull_instance_work_item_buffer(
                device,
                encoder,
                launch_worklist_buffer,
                dispatch_setup,
                instance_work_item_count,
            )
        })
    }
}
