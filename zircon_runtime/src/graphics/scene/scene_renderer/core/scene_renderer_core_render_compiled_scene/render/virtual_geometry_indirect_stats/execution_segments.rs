use std::collections::HashSet;

use crate::core::framework::render::{
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExecutionState,
};
use crate::graphics::scene::scene_renderer::mesh::MeshDraw;

pub(super) fn collect_execution_segments(
    indirect_execution_draws: &[&MeshDraw],
) -> Vec<RenderVirtualGeometryExecutionSegment> {
    indirect_execution_draws
        .iter()
        .enumerate()
        .map(|(draw_index, draw)| draw.virtual_geometry_execution_segment(draw_index as u32))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ExecutionSegmentKey {
    instance_index: u32,
    entity: u64,
    page_id: u32,
    cluster_start_ordinal: u32,
    cluster_span_count: u32,
    cluster_total_count: u32,
    submission_slot: u32,
    state: u32,
    lineage_depth: u32,
    lod_level: u8,
    frontier_rank: u32,
}

#[derive(Default)]
pub(super) struct ExecutionSegmentSummary {
    segment_count: u32,
    page_count: u32,
    resident_segment_count: u32,
    pending_segment_count: u32,
    missing_segment_count: u32,
    repeated_draw_count: u32,
}

impl ExecutionSegmentSummary {
    fn new(
        segment_count: u32,
        page_count: u32,
        resident_segment_count: u32,
        pending_segment_count: u32,
        missing_segment_count: u32,
        repeated_draw_count: u32,
    ) -> Self {
        Self {
            segment_count,
            page_count,
            resident_segment_count,
            pending_segment_count,
            missing_segment_count,
            repeated_draw_count,
        }
    }

    pub(super) fn segment_count(&self) -> u32 {
        self.segment_count
    }

    pub(super) fn page_count(&self) -> u32 {
        self.page_count
    }

    pub(super) fn resident_segment_count(&self) -> u32 {
        self.resident_segment_count
    }

    pub(super) fn pending_segment_count(&self) -> u32 {
        self.pending_segment_count
    }

    pub(super) fn missing_segment_count(&self) -> u32 {
        self.missing_segment_count
    }

    pub(super) fn repeated_draw_count(&self) -> u32 {
        self.repeated_draw_count
    }
}

pub(super) fn execution_segment_summary(
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
    indirect_execution_draw_count: u32,
) -> ExecutionSegmentSummary {
    let mut segments = HashSet::new();
    let mut pages = HashSet::new();
    let mut resident_segment_count = 0;
    let mut pending_segment_count = 0;
    let mut missing_segment_count = 0;

    for segment in execution_segments {
        let key = ExecutionSegmentKey::from(segment);
        if segments.insert(key) {
            pages.insert(segment.page_id);
            match segment.state {
                RenderVirtualGeometryExecutionState::Resident => resident_segment_count += 1,
                RenderVirtualGeometryExecutionState::PendingUpload => pending_segment_count += 1,
                RenderVirtualGeometryExecutionState::Missing => missing_segment_count += 1,
            }
        }
    }

    let segment_count = segments.len() as u32;
    ExecutionSegmentSummary::new(
        segment_count,
        pages.len() as u32,
        resident_segment_count,
        pending_segment_count,
        missing_segment_count,
        indirect_execution_draw_count.saturating_sub(segment_count),
    )
}

impl From<&RenderVirtualGeometryExecutionSegment> for ExecutionSegmentKey {
    fn from(segment: &RenderVirtualGeometryExecutionSegment) -> Self {
        Self {
            instance_index: segment.instance_index.unwrap_or(u32::MAX),
            entity: segment.entity,
            page_id: segment.page_id,
            cluster_start_ordinal: segment.cluster_start_ordinal,
            cluster_span_count: segment.cluster_span_count,
            cluster_total_count: segment.cluster_total_count,
            submission_slot: segment.submission_slot.unwrap_or(u32::MAX),
            state: encode_execution_state(segment.state),
            lineage_depth: segment.lineage_depth,
            lod_level: segment.lod_level,
            frontier_rank: segment.frontier_rank,
        }
    }
}

fn encode_execution_state(state: RenderVirtualGeometryExecutionState) -> u32 {
    match state {
        RenderVirtualGeometryExecutionState::Resident => 0,
        RenderVirtualGeometryExecutionState::PendingUpload => 1,
        RenderVirtualGeometryExecutionState::Missing => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::{execution_segment_summary, RenderVirtualGeometryExecutionSegment};
    use crate::core::framework::render::RenderVirtualGeometryExecutionState;

    #[test]
    fn execution_segment_summary_counts_unique_segments_by_execution_projection() {
        let segments = vec![
            execution_segment(0, 10, RenderVirtualGeometryExecutionState::Resident),
            execution_segment(1, 10, RenderVirtualGeometryExecutionState::Resident),
            execution_segment(2, 11, RenderVirtualGeometryExecutionState::PendingUpload),
        ];

        let summary = execution_segment_summary(&segments, segments.len() as u32);

        assert_eq!(summary.segment_count(), 2);
        assert_eq!(summary.page_count(), 2);
        assert_eq!(summary.resident_segment_count(), 1);
        assert_eq!(summary.pending_segment_count(), 1);
        assert_eq!(summary.missing_segment_count(), 0);
        assert_eq!(summary.repeated_draw_count(), 1);
    }

    fn execution_segment(
        original_index: u32,
        page_id: u32,
        state: RenderVirtualGeometryExecutionState,
    ) -> RenderVirtualGeometryExecutionSegment {
        RenderVirtualGeometryExecutionSegment {
            original_index,
            instance_index: Some(1),
            entity: 42,
            page_id,
            draw_ref_index: original_index,
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
        }
    }
}
