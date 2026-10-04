use std::collections::HashSet;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryExecutionDraw, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState,
};

pub(super) fn collect_execution_segments(
    indirect_execution_draws: &[&RenderVirtualGeometryExecutionDraw],
) -> Vec<RenderVirtualGeometryExecutionSegment> {
    indirect_execution_draws
        .iter()
        .enumerate()
        .map(|(draw_index, draw)| RenderVirtualGeometryExecutionSegment {
            original_index: draw_index as u32,
            ..draw.execution_segment.clone()
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ExecutionSegmentKey {
    instance_index: u32,
    entity: u64,
    stable_instance_key: u64,
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

// 统计按完整执行段身份去重，保留状态和提交槽差异；重复 draw 数由原始数量减去唯一段数。
pub(super) fn execution_segment_summary(
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
    indirect_execution_draw_count: u32,
) -> ExecutionSegmentSummary {
    let mut segments = HashSet::with_capacity(execution_segments.len());
    let mut pages = HashSet::with_capacity(execution_segments.len());
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
            stable_instance_key: segment.stable_instance_key_or_legacy(),
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
#[path = "tests/execution_segments.rs"]
mod tests;
