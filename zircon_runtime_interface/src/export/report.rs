use serde::{Deserialize, Serialize};

use super::{ExportStage, ExportStageIo};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ExportStageStatus {
    Passed,
    Skipped,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportStageRecord {
    pub stage: ExportStage,
    pub io: ExportStageIo,
    pub status: ExportStageStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportPipelineReport {
    pub stages: Vec<ExportStageRecord>,
}

impl ExportPipelineReport {
    pub fn record(&self, stage: ExportStage) -> Option<&ExportStageRecord> {
        self.stages.iter().find(|record| record.stage == stage)
    }
}
