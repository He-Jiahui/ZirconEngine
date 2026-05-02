#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryIndirectStatsStoreParts;
use crate::virtual_geometry::types::VirtualGeometryPrepareClusterState;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_indirect_segments_with_instances(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Vec<(
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
)> {
    parts
        .execution_segments
        .iter()
        .map(|segment| {
            (
                segment.instance_index,
                segment.entity,
                segment.cluster_start_ordinal,
                segment.cluster_span_count,
                segment.cluster_total_count,
                segment.page_id,
                segment.submission_slot.unwrap_or(u32::MAX),
                decode_execution_state(segment.state),
                segment.lineage_depth,
                u32::from(segment.lod_level),
                segment.frontier_rank,
                segment.submission_index.unwrap_or(u32::MAX),
            )
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn read_virtual_geometry_indirect_segments(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Vec<(
    u32,
    u32,
    u32,
    u32,
    u32,
    VirtualGeometryPrepareClusterState,
    u32,
    u32,
    u32,
)> {
    read_virtual_geometry_indirect_segments_with_entities(parts)
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
}

#[cfg(test)]
pub(crate) fn read_virtual_geometry_indirect_segments_with_entities(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Vec<(
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
)> {
    read_virtual_geometry_indirect_segments_with_instances(parts)
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
