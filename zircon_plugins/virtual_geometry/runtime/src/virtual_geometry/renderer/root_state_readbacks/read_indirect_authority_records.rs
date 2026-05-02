#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryIndirectStatsStoreParts;
use crate::virtual_geometry::types::VirtualGeometryPrepareClusterState;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState;

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

#[cfg(test)]
pub(crate) fn read_virtual_geometry_indirect_authority_records(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Vec<VirtualGeometryIndirectAuthorityRecord> {
    authority_records_from_segments(&parts.execution_segments)
}

#[cfg(test)]
pub(crate) fn read_virtual_geometry_indirect_execution_authority_records(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Vec<VirtualGeometryIndirectAuthorityRecord> {
    authority_records_from_segments(&parts.execution_segments)
}

#[cfg(test)]
fn authority_records_from_segments(
    segments: &[zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionSegment],
) -> Vec<VirtualGeometryIndirectAuthorityRecord> {
    segments
        .iter()
        .map(|segment| {
            VirtualGeometryIndirectAuthorityRecord::new(
                segment.draw_ref_index,
                segment.instance_index,
                segment.entity,
                segment.page_id,
                segment.cluster_start_ordinal,
                segment.cluster_span_count,
                segment.cluster_total_count,
                segment.submission_slot.unwrap_or(u32::MAX),
                decode_execution_state(segment.state),
                segment.lineage_depth,
                u32::from(segment.lod_level),
                segment.frontier_rank,
                segment.submission_index.unwrap_or(u32::MAX),
                segment.draw_ref_rank.unwrap_or(u32::MAX),
            )
        })
        .collect()
}

#[cfg(test)]
fn decode_execution_state(
    state: RenderVirtualGeometryExecutionState,
) -> VirtualGeometryPrepareClusterState {
    match state {
        RenderVirtualGeometryExecutionState::Resident => {
            VirtualGeometryPrepareClusterState::Resident
        }
        RenderVirtualGeometryExecutionState::PendingUpload => {
            VirtualGeometryPrepareClusterState::PendingUpload
        }
        RenderVirtualGeometryExecutionState::Missing => VirtualGeometryPrepareClusterState::Missing,
    }
}
