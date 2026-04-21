#[cfg(test)]
use crate::graphics::types::{GraphicsError, VirtualGeometryPrepareClusterState};

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryIndirectExecutionSegmentRecord {
    pub(crate) instance_index: Option<u32>,
    pub(crate) entity: u64,
    pub(crate) cluster_start_ordinal: u32,
    pub(crate) cluster_span_count: u32,
    pub(crate) cluster_total_count: u32,
    pub(crate) page_id: u32,
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
    pub(crate) fn read_last_virtual_geometry_indirect_execution_segments_with_entities(
        &self,
    ) -> Result<Vec<VirtualGeometryIndirectExecutionSegmentRecord>, GraphicsError> {
        let execution_authority_records =
            self.read_last_virtual_geometry_indirect_execution_authority_records()?;
        if !execution_authority_records.is_empty() {
            return Ok(execution_authority_records
                .into_iter()
                .map(|record| VirtualGeometryIndirectExecutionSegmentRecord {
                    instance_index: record.instance_index,
                    entity: record.entity,
                    cluster_start_ordinal: record.cluster_start_ordinal,
                    cluster_span_count: record.cluster_span_count,
                    cluster_total_count: record.cluster_total_count,
                    page_id: record.page_id,
                    submission_slot: record.submission_slot,
                    state: record.state,
                    lineage_depth: record.lineage_depth,
                    lod_level: record.lod_level,
                    frontier_rank: record.frontier_rank,
                    submission_index: record.submission_index,
                    draw_ref_rank: record.draw_ref_rank,
                })
                .collect());
        }
        let authority_records = self.read_last_virtual_geometry_indirect_authority_records()?;
        if authority_records.is_empty() {
            return Ok(Vec::new());
        }
        let authority_by_draw_ref_index = authority_records
            .into_iter()
            .map(|record| {
                (
                    record.draw_ref_index,
                    VirtualGeometryIndirectExecutionSegmentRecord {
                        instance_index: record.instance_index,
                        entity: record.entity,
                        cluster_start_ordinal: record.cluster_start_ordinal,
                        cluster_span_count: record.cluster_span_count,
                        cluster_total_count: record.cluster_total_count,
                        page_id: record.page_id,
                        submission_slot: record.submission_slot,
                        state: record.state,
                        lineage_depth: record.lineage_depth,
                        lod_level: record.lod_level,
                        frontier_rank: record.frontier_rank,
                        submission_index: record.submission_index,
                        draw_ref_rank: record.draw_ref_rank,
                    },
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        Ok(self
            .read_last_virtual_geometry_indirect_execution_draw_ref_indices()?
            .into_iter()
            .filter_map(|draw_ref_index| authority_by_draw_ref_index.get(&draw_ref_index).copied())
            .collect())
    }
}
