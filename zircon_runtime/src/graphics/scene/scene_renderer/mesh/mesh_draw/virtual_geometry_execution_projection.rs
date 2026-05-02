use crate::core::framework::render::{
    RenderVirtualGeometryExecutionDraw, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState,
};

use super::{MeshDraw, VirtualGeometrySubmissionDetail};

const INDIRECT_ARGS_STRIDE_BYTES: u64 = (std::mem::size_of::<u32>() as u64) * 5;

impl MeshDraw {
    pub(crate) fn virtual_geometry_execution_draw(
        &self,
        original_index: u32,
        draw_index: usize,
    ) -> RenderVirtualGeometryExecutionDraw {
        RenderVirtualGeometryExecutionDraw {
            indirect_args_buffer: self.indirect_args_buffer.clone(),
            indirect_args_offset: self.indirect_args_offset,
            uses_indirect_draw: self.indirect_args_buffer.is_some(),
            execution_selection_key: self.virtual_geometry_execution_selection_key(),
            execution_segment: self.virtual_geometry_execution_segment(original_index),
            submission_order_record: self.virtual_geometry_submission_order_record(),
            draw_submission_record: self.virtual_geometry_draw_submission_record(draw_index),
            draw_submission_token_record: self
                .virtual_geometry_draw_submission_token_record(draw_index),
            execution_draw_ref_index: self.virtual_geometry_execution_draw_ref_index(),
        }
    }

    pub(crate) fn virtual_geometry_execution_draw_ref_index(&self) -> u32 {
        execution_draw_ref_index(
            self.virtual_geometry_submission_detail,
            self.indirect_args_offset,
        )
    }

    pub(crate) fn virtual_geometry_execution_selection_key(&self) -> Option<(u64, u32)> {
        let detail = self.virtual_geometry_submission_detail?;
        Some((detail.entity(), detail.submission_index()))
    }

    pub(crate) fn virtual_geometry_execution_segment(
        &self,
        original_index: u32,
    ) -> RenderVirtualGeometryExecutionSegment {
        let fallback_key = self.virtual_geometry_submission_key.unwrap_or((0, 0));
        let detail = self.virtual_geometry_submission_detail;
        let page_id = detail
            .map(|detail| detail.page_id())
            .unwrap_or(fallback_key.1);
        RenderVirtualGeometryExecutionSegment {
            original_index,
            instance_index: detail.and_then(|detail| detail.instance_index()),
            entity: detail
                .map(|detail| detail.entity())
                .unwrap_or(fallback_key.0),
            page_id,
            draw_ref_index: self.virtual_geometry_execution_draw_ref_index(),
            submission_index: detail.map(|detail| detail.submission_index()),
            draw_ref_rank: detail.map(|detail| detail.draw_ref_rank()),
            cluster_start_ordinal: detail
                .map(|detail| detail.cluster_start_ordinal())
                .unwrap_or_default(),
            cluster_span_count: detail
                .map(|detail| detail.cluster_span_count())
                .unwrap_or(1),
            cluster_total_count: detail
                .map(|detail| detail.cluster_total_count())
                .unwrap_or(1),
            submission_slot: detail.and_then(|detail| detail.submission_slot()),
            state: detail
                .map(|detail| detail.state())
                .unwrap_or(RenderVirtualGeometryExecutionState::Resident),
            lineage_depth: detail
                .map(|detail| detail.lineage_depth())
                .unwrap_or_default(),
            lod_level: detail.map(|detail| detail.lod_level()).unwrap_or_default(),
            frontier_rank: detail
                .map(|detail| detail.frontier_rank())
                .unwrap_or_default(),
        }
    }

    pub(crate) fn virtual_geometry_submission_order_record(
        &self,
    ) -> Option<(Option<u32>, u64, u32)> {
        let (entity, page_id) = self.virtual_geometry_submission_key?;
        Some((
            self.virtual_geometry_submission_detail
                .and_then(|detail| detail.instance_index()),
            entity,
            page_id,
        ))
    }

    pub(crate) fn virtual_geometry_draw_submission_record(
        &self,
        draw_index: usize,
    ) -> Option<(u64, u32, u32, usize)> {
        let (entity, page_id) = self.virtual_geometry_submission_key?;
        Some((
            entity,
            page_id,
            self.virtual_geometry_execution_draw_ref_index(),
            draw_index,
        ))
    }

    pub(crate) fn virtual_geometry_draw_submission_token_record(
        &self,
        draw_index: usize,
    ) -> Option<(u64, u32, u32, u32, usize)> {
        self.virtual_geometry_submission_detail.map(|detail| {
            (
                detail.entity(),
                detail.page_id(),
                detail.submission_index(),
                detail.draw_ref_rank(),
                draw_index,
            )
        })
    }
}

fn execution_draw_ref_index(
    submission_detail: Option<VirtualGeometrySubmissionDetail>,
    indirect_args_offset: u64,
) -> u32 {
    submission_detail
        .map(|detail| detail.draw_ref_index())
        .unwrap_or_else(|| (indirect_args_offset / INDIRECT_ARGS_STRIDE_BYTES) as u32)
}

#[cfg(test)]
mod tests {
    use super::{execution_draw_ref_index, VirtualGeometrySubmissionDetail};

    #[test]
    fn execution_draw_ref_index_prefers_explicit_submission_detail_source() {
        let submission_detail = VirtualGeometrySubmissionDetail::new(
            Some(3),
            42,
            300,
            7,
            2,
            9,
            3,
            1,
            4,
            Some(5),
            crate::core::framework::render::RenderVirtualGeometryExecutionState::Resident,
            2,
            1,
            6,
        );

        assert_eq!(
            execution_draw_ref_index(Some(submission_detail), 3 * super::INDIRECT_ARGS_STRIDE_BYTES),
            9,
            "expected execution ownership to keep the authoritative draw-ref index emitted by the shared submission truth instead of reconstructing it from indirect args offsets"
        );
    }

    #[test]
    fn execution_draw_ref_index_falls_back_to_indirect_args_offset_stride() {
        assert_eq!(
            execution_draw_ref_index(None, 4 * super::INDIRECT_ARGS_STRIDE_BYTES),
            4,
            "expected offset-based draw-ref recovery to remain available when explicit authoritative draw-ref truth is absent"
        );
    }
}
