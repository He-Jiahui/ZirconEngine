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

/// 一次导出尝试的阶段回执，持久化后可作为增量恢复的输入。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportPipelineReport {
    pub stages: Vec<ExportStageRecord>,
}

impl ExportPipelineReport {
    /// 按阶段取得首条记录；恢复调用方仍需核对状态、指纹与磁盘产物。
    // TODO: [CR-EXPORT-0002] 外部反序列化报告可含重复阶段；需确定拒绝重复项还是定义首条优先语义，
    // 避免恢复结果取决于记录顺序。
    pub fn record(&self, stage: ExportStage) -> Option<&ExportStageRecord> {
        self.stages.iter().find(|record| record.stage == stage)
    }
}
