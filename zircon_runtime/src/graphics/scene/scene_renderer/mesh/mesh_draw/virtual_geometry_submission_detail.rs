use crate::core::framework::render::RenderVirtualGeometryExecutionState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometrySubmissionDetail {
    instance_index: Option<u32>,
    entity: u64,
    page_id: u32,
    submission_index: u32,
    draw_ref_rank: u32,
    draw_ref_index: u32,
    cluster_start_ordinal: u32,
    cluster_span_count: u32,
    cluster_total_count: u32,
    submission_slot: Option<u32>,
    state: RenderVirtualGeometryExecutionState,
    lineage_depth: u32,
    lod_level: u8,
    frontier_rank: u32,
}

impl VirtualGeometrySubmissionDetail {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        instance_index: Option<u32>,
        entity: u64,
        page_id: u32,
        submission_index: u32,
        draw_ref_rank: u32,
        draw_ref_index: u32,
        cluster_start_ordinal: u32,
        cluster_span_count: u32,
        cluster_total_count: u32,
        submission_slot: Option<u32>,
        state: RenderVirtualGeometryExecutionState,
        lineage_depth: u32,
        lod_level: u8,
        frontier_rank: u32,
    ) -> Self {
        Self {
            instance_index,
            entity,
            page_id,
            submission_index,
            draw_ref_rank,
            draw_ref_index,
            cluster_start_ordinal,
            cluster_span_count,
            cluster_total_count,
            submission_slot,
            state,
            lineage_depth,
            lod_level,
            frontier_rank,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn instance_index(self) -> Option<u32> {
        self.instance_index
    }

    pub(crate) fn entity(self) -> u64 {
        self.entity
    }

    pub(crate) fn page_id(self) -> u32 {
        self.page_id
    }

    #[allow(dead_code)]
    pub(crate) fn submission_index(self) -> u32 {
        self.submission_index
    }

    #[allow(dead_code)]
    pub(crate) fn draw_ref_rank(self) -> u32 {
        self.draw_ref_rank
    }

    pub(crate) fn draw_ref_index(self) -> u32 {
        self.draw_ref_index
    }

    #[allow(dead_code)]
    pub(crate) fn cluster_start_ordinal(self) -> u32 {
        self.cluster_start_ordinal
    }

    #[allow(dead_code)]
    pub(crate) fn cluster_span_count(self) -> u32 {
        self.cluster_span_count
    }

    #[allow(dead_code)]
    pub(crate) fn cluster_total_count(self) -> u32 {
        self.cluster_total_count
    }

    #[allow(dead_code)]
    pub(crate) fn submission_slot(self) -> Option<u32> {
        self.submission_slot
    }

    #[allow(dead_code)]
    pub(crate) fn state(self) -> RenderVirtualGeometryExecutionState {
        self.state
    }

    #[allow(dead_code)]
    pub(crate) fn lineage_depth(self) -> u32 {
        self.lineage_depth
    }

    #[allow(dead_code)]
    pub(crate) fn lod_level(self) -> u8 {
        self.lod_level
    }

    #[allow(dead_code)]
    pub(crate) fn frontier_rank(self) -> u32 {
        self.frontier_rank
    }
}
