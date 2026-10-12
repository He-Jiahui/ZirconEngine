use std::sync::Arc;

use zircon_runtime::core::framework::render::RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot;

use crate::virtual_geometry::renderer::VirtualGeometryGpuResources;

impl VirtualGeometryGpuResources {
    pub(in crate::virtual_geometry::renderer) fn create_virtual_geometry_node_and_cluster_cull_instance_work_item_buffer(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        launch_worklist_buffer: Option<&Arc<wgpu::Buffer>>,
        dispatch_setup: RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
        instance_work_item_count: u32,
    ) -> Option<Arc<wgpu::Buffer>> {
        self.create_node_and_cluster_cull_instance_work_item_buffer(
            device,
            encoder,
            launch_worklist_buffer,
            dispatch_setup,
            instance_work_item_count,
        )
    }
}
