#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::{GraphicsError, VirtualGeometryPrepareClusterState};

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

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
        self.read_last_virtual_geometry_indirect_segments_with_entities()
            .map(|segments| {
                segments
                    .into_iter()
                    .map(
                        |(
                            _entity,
                            cluster_start_ordinal,
                            cluster_span_count,
                            cluster_total_count,
                            page_id,
                            submission_slot,
                            state,
                            lineage_depth,
                            lod_level,
                            frontier_rank,
                        )| {
                            (
                                cluster_start_ordinal,
                                cluster_span_count,
                                cluster_total_count,
                                page_id,
                                submission_slot,
                                state,
                                lineage_depth,
                                lod_level,
                                frontier_rank,
                            )
                        },
                    )
                    .collect()
            })
    }

    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_segments_with_entities(
        &self,
    ) -> Result<
        Vec<(
            u64,
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
                * 12,
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
                * 12,
        );
        self.backend.queue.submit([encoder.finish()]);
        let words = read_buffer_u32s(
            &self.backend.device,
            &staging,
            (self.last_virtual_geometry_indirect_segment_count as usize) * 12,
        )?;

        Ok(words
            .chunks_exact(12)
            .map(|chunk| {
                (
                    u64::from(chunk[10]) | (u64::from(chunk[11]) << 32),
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
