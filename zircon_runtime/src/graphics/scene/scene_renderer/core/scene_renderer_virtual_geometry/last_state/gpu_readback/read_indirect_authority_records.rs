#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::{GraphicsError, VirtualGeometryPrepareClusterState};

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryIndirectAuthorityRecord {
    pub(crate) draw_ref_index: u32,
    pub(crate) instance_index: Option<u32>,
    pub(crate) entity: u64,
    pub(crate) page_id: u32,
    pub(crate) cluster_start_ordinal: u32,
    pub(crate) cluster_span_count: u32,
    pub(crate) cluster_total_count: u32,
    pub(crate) submission_slot: u32,
    pub(crate) state: VirtualGeometryPrepareClusterState,
    pub(crate) lineage_depth: u32,
    pub(crate) lod_level: u32,
    pub(crate) frontier_rank: u32,
    pub(crate) submission_index: u32,
    pub(crate) draw_ref_rank: u32,
}

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_authority_records(
        &self,
    ) -> Result<Vec<VirtualGeometryIndirectAuthorityRecord>, GraphicsError> {
        read_indirect_authority_records_buffer(
            self,
            self.last_virtual_geometry_indirect_authority_buffer
                .as_deref(),
            self.last_virtual_geometry_indirect_args_count,
            "zircon-vg-indirect-authority-records",
        )
    }

    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_execution_authority_records(
        &self,
    ) -> Result<Vec<VirtualGeometryIndirectAuthorityRecord>, GraphicsError> {
        read_indirect_authority_records_buffer(
            self,
            self.last_virtual_geometry_indirect_execution_authority_buffer
                .as_deref(),
            self.last_virtual_geometry_indirect_draw_count,
            "zircon-vg-indirect-execution-authority-records",
        )
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
fn read_indirect_authority_records_buffer(
    renderer: &SceneRenderer,
    buffer: Option<&wgpu::Buffer>,
    record_count: u32,
    label_prefix: &str,
) -> Result<Vec<VirtualGeometryIndirectAuthorityRecord>, GraphicsError> {
    const AUTHORITY_RECORD_WORD_COUNT: usize = 15;

    let Some(buffer) = buffer else {
        return Ok(Vec::new());
    };
    if record_count == 0 {
        return Ok(Vec::new());
    }

    let staging = renderer
        .backend
        .device
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{label_prefix}-readback")),
            size: (record_count as u64)
                * (std::mem::size_of::<u32>() as u64)
                * AUTHORITY_RECORD_WORD_COUNT as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    let mut encoder =
        renderer
            .backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("{label_prefix}-readback-encoder")),
            });
    encoder.copy_buffer_to_buffer(
        buffer,
        0,
        &staging,
        0,
        (record_count as u64)
            * (std::mem::size_of::<u32>() as u64)
            * AUTHORITY_RECORD_WORD_COUNT as u64,
    );
    renderer.backend.queue.submit([encoder.finish()]);
    let words = read_buffer_u32s(
        &renderer.backend.device,
        &staging,
        (record_count as usize) * AUTHORITY_RECORD_WORD_COUNT,
    )?;

    Ok(words
        .chunks_exact(AUTHORITY_RECORD_WORD_COUNT)
        .map(|chunk| VirtualGeometryIndirectAuthorityRecord {
            draw_ref_index: chunk[0],
            instance_index: decode_instance_index(chunk[1]),
            entity: u64::from(chunk[13]) | (u64::from(chunk[14]) << 32),
            page_id: chunk[5],
            cluster_start_ordinal: chunk[2],
            cluster_span_count: chunk[3],
            cluster_total_count: chunk[4],
            submission_slot: chunk[6],
            state: decode_cluster_state(chunk[7]),
            lineage_depth: chunk[8],
            lod_level: chunk[9],
            frontier_rank: chunk[10],
            submission_index: chunk[11],
            draw_ref_rank: chunk[12],
        })
        .collect::<Vec<VirtualGeometryIndirectAuthorityRecord>>())
}

#[cfg(test)]
fn decode_instance_index(encoded: u32) -> Option<u32> {
    (encoded != u32::MAX).then_some(encoded)
}
