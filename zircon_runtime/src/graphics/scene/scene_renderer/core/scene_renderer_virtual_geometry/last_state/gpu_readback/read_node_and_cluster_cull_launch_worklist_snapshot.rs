#[cfg(test)]
use crate::core::framework::render::{
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
};
#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_launch_worklist_snapshot(
        &self,
    ) -> Result<Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>, GraphicsError>
    {
        let Some(buffer) = self
            .last_virtual_geometry_node_and_cluster_cull_launch_worklist_buffer
            .as_ref()
        else {
            return Ok(None);
        };

        let seed_count =
            self.last_virtual_geometry_node_and_cluster_cull_instance_seed_count as usize;
        let word_count =
            RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot::GPU_HEADER_WORD_COUNT
                + seed_count * RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT;
        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-launch-worklist-readback"),
            size: (word_count * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-node-and-cluster-cull-launch-worklist-readback-encoder"),
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
        Ok(
            RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot::from_packed_words(
                &words,
            ),
        )
    }
}
