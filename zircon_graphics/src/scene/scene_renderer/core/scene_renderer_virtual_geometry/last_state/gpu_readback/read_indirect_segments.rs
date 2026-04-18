#[cfg(test)]
use crate::backend::read_buffer_u32s;
#[cfg(test)]
use crate::types::{GraphicsError, VirtualGeometryPrepareClusterState};

use crate::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_segments(
        &self,
    ) -> Result<
        Vec<(
            u32,
            u32,
            u32,
            u32,
            u32,
            VirtualGeometryPrepareClusterState,
            u32,
            u32,
            u32,
        )>,
        GraphicsError,
    > {
        let Some(buffer) = self.last_virtual_geometry_indirect_segments_buffer.as_ref() else {
            return Ok(Vec::new());
        };
        if self.last_virtual_geometry_indirect_segment_count == 0 {
            return Ok(Vec::new());
        }

        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-indirect-segments-readback"),
            size: (self.last_virtual_geometry_indirect_segment_count as u64)
                * (std::mem::size_of::<u32>() as u64)
                * 9,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-indirect-segments-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (self.last_virtual_geometry_indirect_segment_count as u64)
                * (std::mem::size_of::<u32>() as u64)
                * 9,
        );
        self.backend.queue.submit([encoder.finish()]);
        let words = read_buffer_u32s(
            &self.backend.device,
            &staging,
            (self.last_virtual_geometry_indirect_segment_count as usize) * 9,
        )?;

        Ok(words
            .chunks_exact(9)
            .map(|chunk| {
                (
                    chunk[0],
                    chunk[1],
                    chunk[2],
                    chunk[3],
                    chunk[4],
                    decode_cluster_state(chunk[5]),
                    chunk[6],
                    chunk[7],
                    chunk[8],
                )
            })
            .collect())
    }
}

#[cfg(test)]
fn decode_cluster_state(encoded: u32) -> VirtualGeometryPrepareClusterState {
    match encoded {
        0 => VirtualGeometryPrepareClusterState::Resident,
        1 => VirtualGeometryPrepareClusterState::PendingUpload,
        _ => VirtualGeometryPrepareClusterState::Missing,
    }
}
