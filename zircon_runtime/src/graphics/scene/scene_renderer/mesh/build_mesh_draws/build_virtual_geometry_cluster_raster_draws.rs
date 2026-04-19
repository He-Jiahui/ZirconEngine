use std::collections::HashMap;

use crate::graphics::types::EditorOrRuntimeFrame;

use super::virtual_geometry_cluster_raster_draw::VirtualGeometryClusterRasterDraw;

pub(super) fn build_virtual_geometry_cluster_raster_draws(
    frame: &EditorOrRuntimeFrame,
) -> HashMap<u64, Vec<VirtualGeometryClusterRasterDraw>> {
    let mut draws = HashMap::new();
    let Some(prepare) = frame.virtual_geometry_prepare.as_ref() else {
        return draws;
    };
    for (submission_index, indirect_draw) in
        prepare.unified_indirect_draws().into_iter().enumerate()
    {
        draws
            .entry(indirect_draw.entity)
            .or_default()
            .push(VirtualGeometryClusterRasterDraw {
                submission_index: submission_index as u32,
                page_id: indirect_draw.page_id,
                entity_cluster_start_ordinal: indirect_draw.cluster_start_ordinal as usize,
                entity_cluster_span_count: indirect_draw.cluster_span_count as usize,
                entity_cluster_total_count: indirect_draw.cluster_total_count as usize,
                lineage_depth: indirect_draw.lineage_depth,
                lod_level: indirect_draw.lod_level,
                frontier_rank: indirect_draw.frontier_rank,
                resident_slot: indirect_draw.resident_slot,
                submission_slot: indirect_draw.submission_slot,
                state: indirect_draw.state,
            });
    }

    draws
}
