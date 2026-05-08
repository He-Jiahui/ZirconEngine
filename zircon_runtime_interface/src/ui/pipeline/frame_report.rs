use serde::{Deserialize, Serialize};

use super::{UiPipelineStage, UiPipelineStageCounters, UiPipelineStageReport};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPipelineFrameReport {
    pub frame_index: u64,
    pub stages: Vec<UiPipelineStageReport>,
    pub total_elapsed_micros: u64,
    pub totals: UiPipelineStageCounters,
}

impl UiPipelineFrameReport {
    pub fn from_stage_reports(frame_index: u64, stages: Vec<UiPipelineStageReport>) -> Self {
        let mut report = Self {
            frame_index,
            stages,
            ..Self::default()
        };
        report.recompute_totals();
        report
    }

    pub fn recompute_totals(&mut self) {
        let mut totals = UiPipelineStageCounters::default();
        let mut total_elapsed_micros = 0u64;

        for stage in &self.stages {
            total_elapsed_micros += stage.elapsed_micros;
            totals.add_assign(stage.counters);
        }

        self.total_elapsed_micros = total_elapsed_micros;
        self.totals = totals;
    }

    pub fn stage_report(&self, stage: UiPipelineStage) -> Option<&UiPipelineStageReport> {
        self.stages.iter().find(|report| report.stage == stage)
    }

    pub fn completed_stage_count(&self) -> usize {
        self.stages.iter().filter(|report| !report.skipped).count()
    }

    pub fn missing_required_stages(&self) -> Vec<UiPipelineStage> {
        UiPipelineStage::ordered()
            .iter()
            .copied()
            .filter(|stage| self.stage_report(*stage).is_none())
            .collect()
    }

    pub fn is_complete_ordered(&self) -> bool {
        self.stages.len() == UiPipelineStage::ordered().len()
            && self
                .stages
                .iter()
                .map(|report| report.stage)
                .eq(UiPipelineStage::ordered().iter().copied())
    }

    pub const fn repeated_pointer_move_fast_path_holds(&self, expected_moves: u64) -> bool {
        self.totals.pointer_move_count >= expected_moves
            && self.totals.template_reload_count == 0
            && self.totals.full_layout_count == 0
            && self.totals.layout_node_count == 0
    }
}
