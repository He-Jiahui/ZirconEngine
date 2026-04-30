#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::{GraphicsError, VirtualGeometryPrepareClusterState};

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryIndirectAuthorityRecord {
    draw_ref_index: u32,
    instance_index: Option<u32>,
    entity: u64,
    page_id: u32,
    cluster_start_ordinal: u32,
    cluster_span_count: u32,
    cluster_total_count: u32,
    submission_slot: u32,
    state: VirtualGeometryPrepareClusterState,
    lineage_depth: u32,
    lod_level: u32,
    frontier_rank: u32,
    submission_index: u32,
    draw_ref_rank: u32,
}

#[cfg(test)]
impl VirtualGeometryIndirectAuthorityRecord {
    #[allow(clippy::too_many_arguments)]
    fn new(
        draw_ref_index: u32,
        instance_index: Option<u32>,
        entity: u64,
        page_id: u32,
        cluster_start_ordinal: u32,
        cluster_span_count: u32,
        cluster_total_count: u32,
        submission_slot: u32,
        state: VirtualGeometryPrepareClusterState,
        lineage_depth: u32,
        lod_level: u32,
        frontier_rank: u32,
        submission_index: u32,
        draw_ref_rank: u32,
    ) -> Self {
        Self {
            draw_ref_index,
            instance_index,
            entity,
            page_id,
            cluster_start_ordinal,
            cluster_span_count,
            cluster_total_count,
            submission_slot,
            state,
            lineage_depth,
            lod_level,
            frontier_rank,
            submission_index,
            draw_ref_rank,
        }
    }

    pub(crate) fn draw_ref_index(&self) -> u32 {
        self.draw_ref_index
    }

    pub(crate) fn instance_index(&self) -> Option<u32> {
        self.instance_index
    }

    pub(crate) fn entity(&self) -> u64 {
        self.entity
    }

    pub(crate) fn page_id(&self) -> u32 {
        self.page_id
    }

    pub(crate) fn cluster_start_ordinal(&self) -> u32 {
        self.cluster_start_ordinal
    }

    pub(crate) fn cluster_span_count(&self) -> u32 {
        self.cluster_span_count
    }

    pub(crate) fn cluster_total_count(&self) -> u32 {
        self.cluster_total_count
    }

    pub(crate) fn submission_slot(&self) -> u32 {
        self.submission_slot
    }

    pub(crate) fn state(&self) -> VirtualGeometryPrepareClusterState {
        self.state
    }

    pub(crate) fn lineage_depth(&self) -> u32 {
        self.lineage_depth
    }

    pub(crate) fn lod_level(&self) -> u32 {
        self.lod_level
    }

    pub(crate) fn frontier_rank(&self) -> u32 {
        self.frontier_rank
    }

    pub(crate) fn submission_index(&self) -> u32 {
        self.submission_index
    }

    pub(crate) fn draw_ref_rank(&self) -> u32 {
        self.draw_ref_rank
    }

    pub(crate) fn submission_token(&self) -> u32 {
        (self.submission_index.min(0xffff) << 16) | self.draw_ref_rank.min(0xffff)
    }

    pub(crate) fn execution_record(&self) -> (u32, u64, u32, u32, u32) {
        (
            self.draw_ref_index,
            self.entity,
            self.page_id,
            self.submission_index,
            self.draw_ref_rank,
        )
    }
}

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_authority_records(
        &self,
    ) -> Result<Vec<VirtualGeometryIndirectAuthorityRecord>, GraphicsError> {
        read_indirect_authority_records_buffer(
            self,
            self.advanced_plugin_outputs
                .virtual_geometry_indirect_authority_buffer()
                .as_deref(),
            self.advanced_plugin_outputs
                .virtual_geometry_indirect_args_count(),
            "zircon-vg-indirect-authority-records",
        )
    }

    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_execution_authority_records(
        &self,
    ) -> Result<Vec<VirtualGeometryIndirectAuthorityRecord>, GraphicsError> {
        read_indirect_authority_records_buffer(
            self,
            self.advanced_plugin_outputs
                .virtual_geometry_indirect_execution_authority_buffer()
                .as_deref(),
            self.advanced_plugin_outputs
                .virtual_geometry_indirect_draw_count(),
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
        .map(|chunk| {
            VirtualGeometryIndirectAuthorityRecord::new(
                chunk[0],
                decode_instance_index(chunk[1]),
                u64::from(chunk[13]) | (u64::from(chunk[14]) << 32),
                chunk[5],
                chunk[2],
                chunk[3],
                chunk[4],
                chunk[6],
                decode_cluster_state(chunk[7]),
                chunk[8],
                chunk[9],
                chunk[10],
                chunk[11],
                chunk[12],
            )
        })
        .collect::<Vec<VirtualGeometryIndirectAuthorityRecord>>())
}

#[cfg(test)]
fn decode_instance_index(encoded: u32) -> Option<u32> {
    (encoded != u32::MAX).then_some(encoded)
}
