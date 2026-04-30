#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::{GraphicsError, VirtualGeometryPrepareClusterState};

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_segments_with_instances(
        &self,
    ) -> Result<
        Vec<(
            Option<u32>,
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
            u32,
        )>,
        GraphicsError,
    > {
        const INDIRECT_SEGMENT_WORD_COUNT: usize = 13;

        let Some(buffer) = self
            .advanced_plugin_outputs
            .virtual_geometry_indirect_segments_buffer()
            .as_ref()
        else {
            return Ok(Vec::new());
        };
        let indirect_segment_count = self
            .advanced_plugin_outputs
            .virtual_geometry_indirect_segment_count();
        if indirect_segment_count == 0 {
            return Ok(Vec::new());
        }

        let staging = self.backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-indirect-segments-with-instance-readback"),
            size: (indirect_segment_count as u64)
                * (std::mem::size_of::<u32>() as u64)
                * INDIRECT_SEGMENT_WORD_COUNT as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-vg-indirect-segments-with-instance-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(
            buffer,
            0,
            &staging,
            0,
            (indirect_segment_count as u64)
                * (std::mem::size_of::<u32>() as u64)
                * INDIRECT_SEGMENT_WORD_COUNT as u64,
        );
        self.backend.queue.submit([encoder.finish()]);
        let words = read_buffer_u32s(
            &self.backend.device,
            &staging,
            (indirect_segment_count as usize) * INDIRECT_SEGMENT_WORD_COUNT,
        )?;

        Ok(words
            .chunks_exact(INDIRECT_SEGMENT_WORD_COUNT)
            .map(|chunk| {
                (
                    decode_instance_index(chunk[10]),
                    u64::from(chunk[11]) | (u64::from(chunk[12]) << 32),
                    chunk[0],
                    chunk[1],
                    chunk[2],
                    chunk[3],
                    chunk[4],
                    decode_cluster_state(chunk[5]),
                    chunk[6],
                    chunk[7],
                    chunk[8],
                    chunk[9],
                )
            })
            .collect())
    }

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
        self.read_last_virtual_geometry_indirect_segments_with_instances()
            .map(|segments| {
                segments
                    .into_iter()
                    .map(
                        |(
                            _instance_index,
                            entity,
                            cluster_start_ordinal,
                            cluster_span_count,
                            cluster_total_count,
                            page_id,
                            submission_slot,
                            state,
                            lineage_depth,
                            lod_level,
                            frontier_rank,
                            _submission_index,
                        )| {
                            (
                                entity,
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
}

#[cfg(test)]
fn decode_cluster_state(encoded: u32) -> VirtualGeometryPrepareClusterState {
    match encoded {
        0 => VirtualGeometryPrepareClusterState::Resident,
        1 => VirtualGeometryPrepareClusterState::PendingUpload,
        _ => VirtualGeometryPrepareClusterState::Missing,
    }
}

#[cfg(test)]
fn decode_instance_index(encoded: u32) -> Option<u32> {
    (encoded != u32::MAX).then_some(encoded)
}
