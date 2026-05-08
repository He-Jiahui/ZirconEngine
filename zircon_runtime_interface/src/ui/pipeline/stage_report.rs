use serde::{Deserialize, Serialize};

use super::{UiPipelineDirtyReason, UiPipelineStage, UiPipelineStageCounters};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPipelineStageReport {
    pub stage: UiPipelineStage,
    pub elapsed_micros: u64,
    pub skipped: bool,
    pub dirty_reasons: Vec<UiPipelineDirtyReason>,
    pub counters: UiPipelineStageCounters,
    pub notes: Vec<String>,
}

impl Default for UiPipelineStageReport {
    fn default() -> Self {
        Self {
            stage: UiPipelineStage::InputCollect,
            elapsed_micros: 0,
            skipped: false,
            dirty_reasons: Vec::new(),
            counters: UiPipelineStageCounters::default(),
            notes: Vec::new(),
        }
    }
}

impl UiPipelineStageReport {
    pub fn new(
        stage: UiPipelineStage,
        elapsed_micros: u64,
        dirty_reasons: Vec<UiPipelineDirtyReason>,
        counters: UiPipelineStageCounters,
    ) -> Self {
        Self {
            stage,
            elapsed_micros,
            skipped: false,
            dirty_reasons,
            counters,
            notes: Vec::new(),
        }
    }

    pub fn skipped(stage: UiPipelineStage, dirty_reasons: Vec<UiPipelineDirtyReason>) -> Self {
        Self {
            stage,
            skipped: true,
            dirty_reasons,
            ..Self::default()
        }
    }
}
