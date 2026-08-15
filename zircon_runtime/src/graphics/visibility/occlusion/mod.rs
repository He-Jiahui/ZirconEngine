mod hzb_builder;

pub use hzb_builder::{HzbBuildPlan, HzbBuilder};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HzbOcclusionPhase {
    SingleFrameReproject,
    TwoPhaseRetest,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HzbOcclusionCullReadbackStats {
    pub tested_arg_count: u32,
    pub tested_instance_count: u32,
    pub culled_arg_count: u32,
    pub culled_instance_count: u32,
}

impl HzbOcclusionCullReadbackStats {
    pub const fn new(
        tested_arg_count: u32,
        tested_instance_count: u32,
        culled_arg_count: u32,
        culled_instance_count: u32,
    ) -> Self {
        Self {
            tested_arg_count,
            tested_instance_count,
            culled_arg_count,
            culled_instance_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HzbOcclusionIndirectArgsReadbackSummary {
    pub readback_arg_count: u32,
    pub compacted_draw_count: u32,
    pub zero_instance_arg_count: u32,
    pub remaining_instance_count: u32,
}

impl HzbOcclusionIndirectArgsReadbackSummary {
    pub const fn new(
        readback_arg_count: u32,
        compacted_draw_count: u32,
        zero_instance_arg_count: u32,
        remaining_instance_count: u32,
    ) -> Self {
        Self {
            readback_arg_count,
            compacted_draw_count,
            zero_instance_arg_count,
            remaining_instance_count,
        }
    }

    pub fn add_assign(&mut self, other: Self) {
        self.readback_arg_count = self
            .readback_arg_count
            .saturating_add(other.readback_arg_count);
        self.compacted_draw_count = self
            .compacted_draw_count
            .saturating_add(other.compacted_draw_count);
        self.zero_instance_arg_count = self
            .zero_instance_arg_count
            .saturating_add(other.zero_instance_arg_count);
        self.remaining_instance_count = self
            .remaining_instance_count
            .saturating_add(other.remaining_instance_count);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HzbOcclusionCullReport {
    pub phase: Option<HzbOcclusionPhase>,
    pub candidate_arg_count: u32,
    pub candidate_instance_count: u32,
    pub dispatch_group_count: u32,
    pub dispatched_phase_count: u32,
    pub params_buffer_create_count: u32,
    pub params_upload_byte_count: u64,
    pub bind_group_create_count: u32,
    pub history_available: bool,
    pub readback_stats: Option<HzbOcclusionCullReadbackStats>,
    pub readback_stats_source_frame_index: Option<u64>,
    pub readback_pending_count: u32,
    pub readback_dropped_count: u32,
    pub readback_oldest_pending_age_frames: Option<u64>,
    pub indirect_args_readback: Option<HzbOcclusionIndirectArgsReadbackSummary>,
    pub indirect_args_readback_source_frame_index: Option<u64>,
}

impl HzbOcclusionCullReport {
    pub const fn skipped() -> Self {
        Self {
            phase: None,
            candidate_arg_count: 0,
            candidate_instance_count: 0,
            dispatch_group_count: 0,
            dispatched_phase_count: 0,
            params_buffer_create_count: 0,
            params_upload_byte_count: 0,
            bind_group_create_count: 0,
            history_available: false,
            readback_stats: None,
            readback_stats_source_frame_index: None,
            readback_pending_count: 0,
            readback_dropped_count: 0,
            readback_oldest_pending_age_frames: None,
            indirect_args_readback: None,
            indirect_args_readback_source_frame_index: None,
        }
    }

    pub const fn single_frame_reproject(
        candidate_arg_count: u32,
        candidate_instance_count: u32,
        dispatch_group_count: u32,
        dispatched_phase_count: u32,
        history_available: bool,
    ) -> Self {
        Self {
            phase: Some(HzbOcclusionPhase::SingleFrameReproject),
            candidate_arg_count,
            candidate_instance_count,
            dispatch_group_count,
            dispatched_phase_count,
            params_buffer_create_count: 0,
            params_upload_byte_count: 0,
            bind_group_create_count: 0,
            history_available,
            readback_stats: None,
            readback_stats_source_frame_index: None,
            readback_pending_count: 0,
            readback_dropped_count: 0,
            readback_oldest_pending_age_frames: None,
            indirect_args_readback: None,
            indirect_args_readback_source_frame_index: None,
        }
    }

    pub const fn with_workspace_stats(
        mut self,
        params_buffer_create_count: u32,
        params_upload_byte_count: u64,
        bind_group_create_count: u32,
    ) -> Self {
        self.params_buffer_create_count = params_buffer_create_count;
        self.params_upload_byte_count = params_upload_byte_count;
        self.bind_group_create_count = bind_group_create_count;
        self
    }

    pub const fn with_readback_stats(
        mut self,
        readback_stats: HzbOcclusionCullReadbackStats,
    ) -> Self {
        self.readback_stats = Some(readback_stats);
        self
    }

    pub const fn with_readback_stats_source_frame_index(mut self, source_frame_index: u64) -> Self {
        self.readback_stats_source_frame_index = Some(source_frame_index);
        self
    }

    pub const fn with_readback_queue_diagnostics(
        mut self,
        pending_count: u32,
        dropped_count: u32,
        oldest_pending_age_frames: Option<u64>,
    ) -> Self {
        self.readback_pending_count = pending_count;
        self.readback_dropped_count = dropped_count;
        self.readback_oldest_pending_age_frames = oldest_pending_age_frames;
        self
    }

    pub const fn with_indirect_args_readback(
        mut self,
        indirect_args_readback: HzbOcclusionIndirectArgsReadbackSummary,
    ) -> Self {
        self.indirect_args_readback = Some(indirect_args_readback);
        self
    }

    pub const fn with_indirect_args_readback_source_frame_index(
        mut self,
        source_frame_index: u64,
    ) -> Self {
        self.indirect_args_readback_source_frame_index = Some(source_frame_index);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HzbOcclusionCullReadbackStats, HzbOcclusionCullReport,
        HzbOcclusionIndirectArgsReadbackSummary,
    };

    #[test]
    fn hzb_occlusion_report_preserves_readback_stats() {
        let readback_stats = HzbOcclusionCullReadbackStats::new(6, 42, 2, 18);
        let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
            .with_readback_stats(readback_stats);

        assert_eq!(report.readback_stats, Some(readback_stats));
    }

    #[test]
    fn hzb_occlusion_report_preserves_workspace_churn() {
        let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
            .with_workspace_stats(1, 64, 2);

        assert_eq!(report.params_buffer_create_count, 1);
        assert_eq!(report.params_upload_byte_count, 64);
        assert_eq!(report.bind_group_create_count, 2);
    }

    #[test]
    fn hzb_occlusion_report_preserves_indirect_args_readback_summary() {
        let summary = HzbOcclusionIndirectArgsReadbackSummary::new(6, 4, 2, 24);
        let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
            .with_indirect_args_readback(summary)
            .with_indirect_args_readback_source_frame_index(17);

        assert_eq!(report.indirect_args_readback, Some(summary));
        assert_eq!(report.indirect_args_readback_source_frame_index, Some(17));
    }

    #[test]
    fn hzb_occlusion_report_records_delayed_stats_source_frame() {
        let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
            .with_readback_stats(HzbOcclusionCullReadbackStats::new(6, 42, 2, 18))
            .with_readback_stats_source_frame_index(11);

        assert_eq!(report.readback_stats_source_frame_index, Some(11));
    }

    #[test]
    fn hzb_occlusion_report_preserves_async_readback_queue_diagnostics() {
        let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
            .with_readback_queue_diagnostics(3, 2, Some(4));

        assert_eq!(report.readback_pending_count, 3);
        assert_eq!(report.readback_dropped_count, 2);
        assert_eq!(report.readback_oldest_pending_age_frames, Some(4));
    }

    #[test]
    fn hzb_occlusion_indirect_args_summary_saturates_totals() {
        let mut summary =
            HzbOcclusionIndirectArgsReadbackSummary::new(u32::MAX, u32::MAX, 1, u32::MAX);

        summary.add_assign(HzbOcclusionIndirectArgsReadbackSummary::new(1, 1, 2, 1));

        assert_eq!(summary.readback_arg_count, u32::MAX);
        assert_eq!(summary.compacted_draw_count, u32::MAX);
        assert_eq!(summary.zero_instance_arg_count, 3);
        assert_eq!(summary.remaining_instance_count, u32::MAX);
    }
}
