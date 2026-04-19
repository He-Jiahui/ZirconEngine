#[cfg(test)]
use crate::backend::read_buffer_u32s;
#[cfg(test)]
use crate::types::GraphicsError;

use crate::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_submission_tokens(
        &self,
    ) -> Result<Vec<u32>, GraphicsError> {
        let Some(buffer) = self
            .last_virtual_geometry_indirect_submission_buffer
            .as_ref()
        else {
            return Ok(Vec::new());
        };
        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-indirect-submission-tokens-readback"),
            size: (self.last_virtual_geometry_indirect_args_count as u64)
                * (std::mem::size_of::<u32>() as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-indirect-submission-tokens-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (self.last_virtual_geometry_indirect_args_count as u64)
                * (std::mem::size_of::<u32>() as u64),
        );
        self.backend.queue.submit([encoder.finish()]);
        read_buffer_u32s(
            &self.backend.device,
            &staging,
            self.last_virtual_geometry_indirect_args_count as usize,
        )
    }
}
