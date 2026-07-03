use super::super::{
    PreparedMeshQueueStats, PreparedMeshVirtualGeometryExecutionStats,
    PreparedMeshVirtualGeometryIndirectStats,
};
use crate::core::framework::render::{
    RenderVirtualGeometryExecutionDraw, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState,
};

#[test]
fn prepared_queue_stats_carry_virtual_geometry_indirect_counts() {
    let stats = PreparedMeshQueueStats::default().with_virtual_geometry_indirect_stats(
        PreparedMeshVirtualGeometryIndirectStats {
            draw_count: 3,
            buffer_count: 5,
            args_count: 3,
            segment_count: 2,
        },
    );

    assert_eq!(stats.virtual_geometry_indirect_draw_count, 3);
    assert_eq!(stats.virtual_geometry_indirect_buffer_count, 5);
    assert_eq!(stats.virtual_geometry_indirect_args_count, 3);
    assert_eq!(stats.virtual_geometry_indirect_segment_count, 2);
}

#[test]
fn prepared_queue_stats_carry_virtual_geometry_execution_counts() {
    let execution_stats = PreparedMeshVirtualGeometryExecutionStats::from_execution_draws([
        execution_draw(0, 10, RenderVirtualGeometryExecutionState::Resident),
        execution_draw(1, 10, RenderVirtualGeometryExecutionState::Resident),
        execution_draw(2, 11, RenderVirtualGeometryExecutionState::PendingUpload),
        non_virtual_geometry_indirect_draw(),
    ]);
    let stats =
        PreparedMeshQueueStats::default().with_virtual_geometry_execution_stats(execution_stats);

    assert_eq!(stats.virtual_geometry_execution_draw_count, 3);
    assert_eq!(stats.virtual_geometry_execution_segment_count, 2);
    assert_eq!(stats.virtual_geometry_execution_page_count, 2);
    assert_eq!(stats.virtual_geometry_execution_resident_segment_count, 1);
    assert_eq!(stats.virtual_geometry_execution_pending_segment_count, 1);
    assert_eq!(stats.virtual_geometry_execution_missing_segment_count, 0);
    assert_eq!(stats.virtual_geometry_execution_repeated_draw_count, 1);
}

fn execution_draw(
    draw_ref_index: u32,
    page_id: u32,
    state: RenderVirtualGeometryExecutionState,
) -> RenderVirtualGeometryExecutionDraw {
    RenderVirtualGeometryExecutionDraw {
        indirect_args_buffer_available: true,
        indirect_args_offset: u64::from(draw_ref_index) * 20,
        uses_indirect_draw: true,
        execution_selection_key: Some((42, page_id)),
        execution_segment: RenderVirtualGeometryExecutionSegment {
            original_index: draw_ref_index,
            instance_index: Some(1),
            entity: 42,
            page_id,
            draw_ref_index,
            submission_index: Some(page_id),
            draw_ref_rank: Some(0),
            cluster_start_ordinal: 0,
            cluster_span_count: 1,
            cluster_total_count: 1,
            submission_slot: Some(page_id),
            state,
            lineage_depth: 0,
            lod_level: 0,
            frontier_rank: 0,
        },
        submission_order_record: Some((Some(1), 42, page_id)),
        draw_submission_record: Some((42, page_id, draw_ref_index, draw_ref_index as usize)),
        draw_submission_token_record: Some((42, page_id, page_id, 0, draw_ref_index as usize)),
        execution_draw_ref_index: draw_ref_index,
    }
}

fn non_virtual_geometry_indirect_draw() -> RenderVirtualGeometryExecutionDraw {
    let mut draw = execution_draw(99, 99, RenderVirtualGeometryExecutionState::Resident);
    draw.execution_selection_key = None;
    draw.execution_segment.entity = 0;
    draw
}
