#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem;
#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_instance_work_items(
        &self,
    ) -> Result<Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>, GraphicsError> {
        let work_item_count =
            self.last_virtual_geometry_node_and_cluster_cull_instance_work_item_count as usize;
        let Some(buffer) = self
            .last_virtual_geometry_node_and_cluster_cull_instance_work_item_buffer
            .as_ref()
        else {
            return Ok(Vec::new());
        };
        if work_item_count == 0 {
            return Ok(Vec::new());
        }

        let word_count = work_item_count
            * RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::GPU_WORD_COUNT;
        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-instance-work-item-readback"),
            size: (word_count * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some(
                        "zircon-vg-node-and-cluster-cull-instance-work-item-readback-encoder",
                    ),
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
        Ok(words
            .chunks_exact(RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::GPU_WORD_COUNT)
            .filter_map(RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::from_packed_words)
            .collect())
    }
}
