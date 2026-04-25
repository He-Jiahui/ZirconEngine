#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_page_requests(
        &self,
    ) -> Result<Vec<u32>, GraphicsError> {
        let page_request_count =
            self.last_virtual_geometry_node_and_cluster_cull_page_request_count as usize;
        let Some(buffer) = self
            .last_virtual_geometry_node_and_cluster_cull_page_request_buffer
            .as_ref()
        else {
            return Ok(Vec::new());
        };
        if page_request_count == 0 {
            return Ok(Vec::new());
        }

        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-page-request-readback"),
            size: (page_request_count * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-node-and-cluster-cull-page-request-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (page_request_count * std::mem::size_of::<u32>()) as u64,
        );
        self.backend.queue.submit([encoder.finish()]);

        read_buffer_u32s(&self.backend.device, &staging, page_request_count)
    }
}
