#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryIndirectStatsStoreParts;
#[cfg(test)]
use super::read_indirect_authority_records::{
    read_virtual_geometry_indirect_authority_records,
    read_virtual_geometry_indirect_execution_authority_records,
};
#[cfg(test)]
use super::read_indirect_execution_indices::read_virtual_geometry_indirect_execution_draw_ref_indices;
use crate::virtual_geometry::types::VirtualGeometryPrepareClusterState;

#[cfg(test)]
use super::read_indirect_authority_records::VirtualGeometryIndirectAuthorityRecord;

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryIndirectExecutionSegmentRecord {
    instance_index: Option<u32>,
    entity: u64,
    cluster_start_ordinal: u32,
    cluster_span_count: u32,
    cluster_total_count: u32,
    page_id: u32,
    submission_slot: u32,
    state: VirtualGeometryPrepareClusterState,
    lineage_depth: u32,
    lod_level: u32,
    frontier_rank: u32,
    submission_index: u32,
    draw_ref_rank: u32,
}

#[cfg(test)]
impl VirtualGeometryIndirectExecutionSegmentRecord {
    #[allow(clippy::too_many_arguments)]
    fn new(
        instance_index: Option<u32>,
        entity: u64,
        cluster_start_ordinal: u32,
        cluster_span_count: u32,
        cluster_total_count: u32,
        page_id: u32,
        submission_slot: u32,
        state: VirtualGeometryPrepareClusterState,
        lineage_depth: u32,
        lod_level: u32,
        frontier_rank: u32,
        submission_index: u32,
        draw_ref_rank: u32,
    ) -> Self {
        Self {
            instance_index,
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
            submission_index,
            draw_ref_rank,
        }
    }

    pub(crate) fn from_authority_record(record: VirtualGeometryIndirectAuthorityRecord) -> Self {
        Self::new(
            record.instance_index(),
            record.entity(),
            record.cluster_start_ordinal(),
            record.cluster_span_count(),
            record.cluster_total_count(),
            record.page_id(),
            record.submission_slot(),
            record.state(),
            record.lineage_depth(),
            record.lod_level(),
            record.frontier_rank(),
            record.submission_index(),
            record.draw_ref_rank(),
        )
    }

    pub(crate) fn instance_index(&self) -> Option<u32> {
        self.instance_index
    }

    pub(crate) fn execution_order_tuple(
        &self,
    ) -> (
        Option<u32>,
        (
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
            u32,
        ),
    ) {
        (
            self.instance_index,
            (
                self.entity,
                self.cluster_start_ordinal,
                self.cluster_span_count,
                self.cluster_total_count,
                self.page_id,
                self.submission_slot,
                self.state,
                self.lineage_depth,
                self.lod_level,
                self.frontier_rank,
                self.submission_index,
                self.draw_ref_rank,
            ),
        )
    }
}

#[cfg(test)]
pub(crate) fn read_virtual_geometry_indirect_execution_segments_with_entities(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Vec<VirtualGeometryIndirectExecutionSegmentRecord> {
    let execution_authority_records =
        read_virtual_geometry_indirect_execution_authority_records(parts);
    if !execution_authority_records.is_empty() {
        return execution_authority_records
            .into_iter()
            .map(VirtualGeometryIndirectExecutionSegmentRecord::from_authority_record)
            .collect();
    }
    let authority_records = read_virtual_geometry_indirect_authority_records(parts);
    if authority_records.is_empty() {
        return Vec::new();
    }
    let authority_by_draw_ref_index = authority_records
        .into_iter()
        .map(|record| {
            (
                record.draw_ref_index(),
                VirtualGeometryIndirectExecutionSegmentRecord::from_authority_record(record),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();
    read_virtual_geometry_indirect_execution_draw_ref_indices(parts)
        .into_iter()
        .filter_map(|draw_ref_index| authority_by_draw_ref_index.get(&draw_ref_index).copied())
        .collect()
}
