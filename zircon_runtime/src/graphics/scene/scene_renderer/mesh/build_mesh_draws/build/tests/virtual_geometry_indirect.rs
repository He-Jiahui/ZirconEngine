use super::{
    execution_segments_by_stable_instance_key, expand_pending_draws_for_execution_segments,
};
use crate::core::framework::render::{
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExecutionState,
};

#[test]
fn execution_segments_keep_same_entity_primitives_partitioned_by_stable_instance_key() {
    let segments = vec![
        execution_segment(41, 41 << 16, 10),
        execution_segment(41, (41 << 16) | 1, 20),
    ];

    let segments_by_stable_instance_key = execution_segments_by_stable_instance_key(&segments);

    assert_eq!(segments_by_stable_instance_key.len(), 2);
    assert_eq!(segments_by_stable_instance_key[&(41 << 16)][0].page_id, 10);
    assert_eq!(
        segments_by_stable_instance_key[&((41 << 16) | 1)][0].page_id,
        20
    );
}

#[test]
fn legacy_execution_segment_key_only_matches_primitive_zero_for_its_entity() {
    let entity = 41;
    let segments = vec![
        execution_segment(entity, 0, 10),
        execution_segment(entity, (entity << 16) | 1, 20),
    ];

    let segments_by_stable_instance_key = execution_segments_by_stable_instance_key(&segments);

    assert_eq!(segments_by_stable_instance_key.len(), 2);
    assert_eq!(
        segments_by_stable_instance_key[&(entity << 16)][0].page_id,
        10
    );
    assert_eq!(
        segments_by_stable_instance_key[&((entity << 16) | 1)][0].page_id,
        20
    );
}

#[test]
fn pending_draw_expansion_keeps_same_entity_primitives_and_legacy_key_isolated() {
    let entity = 41_u64;
    let first_key = entity << 16;
    let second_key = first_key | 1;
    let segments_by_stable_instance_key = execution_segments_by_stable_instance_key(&[
        execution_segment(entity, 0, 10),
        execution_segment(entity, second_key, 20),
    ]);
    let mut pending_draws = vec![
        PendingDrawKey::new(first_key),
        PendingDrawKey::new(second_key),
    ];

    let draw_segments = expand_pending_draws_for_execution_segments(
        &mut pending_draws,
        &segments_by_stable_instance_key,
        |draw| draw.stable_instance_key,
        |draw, segment| draw.attached_segment_page = Some(segment.page_id),
    );

    assert_eq!(pending_draws.len(), 2);
    assert_eq!(pending_draws[0].attached_segment_page, Some(10));
    assert_eq!(pending_draws[1].attached_segment_page, Some(20));
    assert_eq!(draw_segments.len(), 2);
    assert_eq!(
        draw_segments[0].as_ref().map(|segment| segment.page_id),
        Some(10)
    );
    assert_eq!(
        draw_segments[1].as_ref().map(|segment| segment.page_id),
        Some(20)
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingDrawKey {
    stable_instance_key: u64,
    attached_segment_page: Option<u32>,
}

impl PendingDrawKey {
    const fn new(stable_instance_key: u64) -> Self {
        Self {
            stable_instance_key,
            attached_segment_page: None,
        }
    }
}

fn execution_segment(
    entity: u64,
    stable_instance_key: u64,
    page_id: u32,
) -> RenderVirtualGeometryExecutionSegment {
    RenderVirtualGeometryExecutionSegment {
        original_index: 0,
        instance_index: None,
        entity,
        stable_instance_key,
        page_id,
        draw_ref_index: 0,
        submission_index: Some(0),
        draw_ref_rank: Some(0),
        cluster_start_ordinal: 0,
        cluster_span_count: 1,
        cluster_total_count: 1,
        submission_slot: Some(0),
        state: RenderVirtualGeometryExecutionState::Resident,
        lineage_depth: 0,
        lod_level: 0,
        frontier_rank: 0,
    }
}
