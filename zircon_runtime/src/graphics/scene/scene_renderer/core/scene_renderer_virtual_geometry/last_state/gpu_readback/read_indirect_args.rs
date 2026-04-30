#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_args_with_instances(
        &self,
    ) -> Result<Vec<(u32, u32, u32)>, GraphicsError> {
        let Some(buffer) = self
            .advanced_plugin_outputs
            .virtual_geometry_indirect_args_buffer()
            .as_ref()
        else {
            return Ok(Vec::new());
        };
        let indirect_args_count = self
            .advanced_plugin_outputs
            .virtual_geometry_indirect_args_count();
        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-indirect-args-with-instance-readback"),
            size: (indirect_args_count as u64) * (std::mem::size_of::<u32>() as u64) * 5,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-indirect-args-with-instance-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (indirect_args_count as u64) * (std::mem::size_of::<u32>() as u64) * 5,
        );
        self.backend.queue.submit([encoder.finish()]);
        let words = read_buffer_u32s(
            &self.backend.device,
            &staging,
            (indirect_args_count as usize) * 5,
        )?;

        Ok(words
            .chunks_exact(5)
            .map(|chunk| (chunk[2], chunk[0], chunk[4]))
            .collect())
    }

    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_args(
        &self,
    ) -> Result<Vec<(u32, u32)>, GraphicsError> {
        let Some(buffer) = self
            .advanced_plugin_outputs
            .virtual_geometry_indirect_args_buffer()
            .as_ref()
        else {
            return Ok(Vec::new());
        };
        let indirect_args_count = self
            .advanced_plugin_outputs
            .virtual_geometry_indirect_args_count();
        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-indirect-args-readback"),
            size: (indirect_args_count as u64) * (std::mem::size_of::<u32>() as u64) * 5,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-indirect-args-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (indirect_args_count as u64) * (std::mem::size_of::<u32>() as u64) * 5,
        );
        self.backend.queue.submit([encoder.finish()]);
        let words = read_buffer_u32s(
            &self.backend.device,
            &staging,
            (indirect_args_count as usize) * 5,
        )?;

        Ok(words
            .chunks_exact(5)
            .map(|chunk| (chunk[2], chunk[0]))
            .collect())
    }
}
