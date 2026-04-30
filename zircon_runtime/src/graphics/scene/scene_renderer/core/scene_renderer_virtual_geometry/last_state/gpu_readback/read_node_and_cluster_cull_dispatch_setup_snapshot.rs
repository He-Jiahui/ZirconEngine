#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot;
#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_dispatch_setup_snapshot(
        &self,
    ) -> Result<Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>, GraphicsError>
    {
        let Some(buffer) = self
            .advanced_plugin_outputs
            .virtual_geometry_node_and_cluster_cull_dispatch_setup_buffer()
            .as_ref()
        else {
            return Ok(None);
        };

        let word_count =
            RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::GPU_WORD_COUNT;
        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-dispatch-setup-readback"),
            size: (word_count * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-node-and-cluster-cull-dispatch-setup-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (word_count * std::mem::size_of::<u32>()) as u64,
        );
        self.backend.queue.submit([encoder.finish()]);

        let words = read_buffer_u32s(&self.backend.device, &staging, word_count)?;
        Ok(RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::from_packed_words(&words))
    }
}
