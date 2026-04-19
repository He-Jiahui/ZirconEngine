#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_execution_draw_ref_indices(
        &self,
    ) -> Result<Vec<u32>, GraphicsError> {
        let Some(buffer) = self
            .last_virtual_geometry_indirect_execution_buffer
            .as_ref()
        else {
            return Ok(self
                .read_last_virtual_geometry_indirect_execution_records()?
                .into_iter()
                .map(
                    |(draw_ref_index, _entity, _page_id, _submission_index, _draw_ref_rank)| {
                        draw_ref_index
                    },
                )
                .collect());
        };
        if self.last_virtual_geometry_indirect_draw_count == 0 {
            return Ok(Vec::new());
        }

        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-indirect-execution-draw-ref-readback"),
            size: (self.last_virtual_geometry_indirect_draw_count as u64)
                * (std::mem::size_of::<u32>() as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-indirect-execution-draw-ref-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (self.last_virtual_geometry_indirect_draw_count as u64)
                * (std::mem::size_of::<u32>() as u64),
        );
        self.backend.queue.submit([encoder.finish()]);

        read_buffer_u32s(
            &self.backend.device,
            &staging,
            self.last_virtual_geometry_indirect_draw_count as usize,
        )
    }
}
