#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_visbuffer64_words(
        &self,
    ) -> Result<(u64, Vec<u64>), GraphicsError> {
        let clear_value = self.last_virtual_geometry_visbuffer64_clear_value;
        let entry_count = self.last_virtual_geometry_visbuffer64_entry_count as usize;
        let Some(buffer) = self.last_virtual_geometry_visbuffer64_buffer.as_ref() else {
            return Ok((clear_value, Vec::new()));
        };
        if entry_count == 0 {
            return Ok((clear_value, Vec::new()));
        }

        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-visbuffer64-readback"),
            size: (entry_count * std::mem::size_of::<u64>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-visbuffer64-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (entry_count * std::mem::size_of::<u64>()) as u64,
        );
        self.backend.queue.submit([encoder.finish()]);

        let words = read_buffer_u32s(&self.backend.device, &staging, entry_count * 2)?;
        Ok((
            clear_value,
            words
                .chunks_exact(2)
                .map(|chunk| u64::from(chunk[0]) | (u64::from(chunk[1]) << 32))
                .collect(),
        ))
    }
}
