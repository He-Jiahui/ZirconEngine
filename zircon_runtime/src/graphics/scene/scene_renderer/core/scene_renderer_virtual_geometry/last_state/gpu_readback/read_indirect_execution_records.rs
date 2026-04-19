#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_execution_records(
        &self,
    ) -> Result<Vec<(u32, u64, u32, u32, u32)>, GraphicsError> {
        const EXECUTION_RECORD_WORD_COUNT: usize = 6;

        let Some(buffer) = self
            .last_virtual_geometry_indirect_execution_records_buffer
            .as_ref()
        else {
            return Ok(Vec::new());
        };
        if self.last_virtual_geometry_indirect_draw_count == 0 {
            return Ok(Vec::new());
        }

        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-indirect-execution-records-readback"),
            size: (self.last_virtual_geometry_indirect_draw_count as u64)
                * (std::mem::size_of::<u32>() as u64)
                * EXECUTION_RECORD_WORD_COUNT as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-indirect-execution-records-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (self.last_virtual_geometry_indirect_draw_count as u64)
                * (std::mem::size_of::<u32>() as u64)
                * EXECUTION_RECORD_WORD_COUNT as u64,
        );
        self.backend.queue.submit([encoder.finish()]);
        let words = read_buffer_u32s(
            &self.backend.device,
            &staging,
            (self.last_virtual_geometry_indirect_draw_count as usize) * EXECUTION_RECORD_WORD_COUNT,
        )?;

        Ok(words
            .chunks_exact(EXECUTION_RECORD_WORD_COUNT)
            .map(|chunk| {
                (
                    chunk[0],
                    u64::from(chunk[4]) | (u64::from(chunk[5]) << 32),
                    chunk[1],
                    chunk[2],
                    chunk[3],
                )
            })
            .collect())
    }
}
