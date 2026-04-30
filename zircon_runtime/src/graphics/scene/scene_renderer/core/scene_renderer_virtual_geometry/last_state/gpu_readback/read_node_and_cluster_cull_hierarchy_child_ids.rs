#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_hierarchy_child_ids(
        &self,
    ) -> Result<Vec<u32>, GraphicsError> {
        let child_id_count = self
            .advanced_plugin_outputs
            .virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count()
            as usize;
        let Some(buffer) = self
            .advanced_plugin_outputs
            .virtual_geometry_node_and_cluster_cull_hierarchy_child_id_buffer()
            .as_ref()
        else {
            return Ok(Vec::new());
        };
        if child_id_count == 0 {
            return Ok(Vec::new());
        }

        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-hierarchy-child-id-readback"),
            size: (child_id_count * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some(
                        "zircon-vg-node-and-cluster-cull-hierarchy-child-id-readback-encoder",
                    ),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (child_id_count * std::mem::size_of::<u32>()) as u64,
        );
        self.backend.queue.submit([encoder.finish()]);

        read_buffer_u32s(&self.backend.device, &staging, child_id_count)
    }
}
