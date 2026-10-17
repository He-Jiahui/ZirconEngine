//! 阶段报告是 Runtime 重建诊断交给帧报告和 Editor 调试视图的共享 DTO。

use serde::{Deserialize, Serialize};

use super::{UiPipelineDirtyReason, UiPipelineStage, UiPipelineStageCounters};

/// skipped 表示本报告未测得该阶段的重建工作，不代表整帧未执行对应行为。
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
    /// 字段缺省基线，不代表输入收集阶段曾执行或已完成测量。
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
    /// 组装未标为跳过的阶段观测，不运行该阶段。
    ///
    /// # 参数
    /// - `stage`：观测对应的阶段标识。
    /// - `elapsed_micros`：调用方提供的耗时，单位微秒。
    /// - `dirty_reasons`：移入报告的原因列表，本函数不去重。
    /// - `counters`：调用方已采集的计数值。
    ///
    /// # 返回
    /// 说明列表为空的独立报告；原因向量归返回值所有。
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

    /// 为本次未测得重建工作的阶段保留一行报告。
    ///
    /// # 参数与返回
    /// `stage` 指定阶段，`dirty_reasons` 的所有权转入返回值。
    /// 返回行标为跳过，耗时、计数及说明列表沿用缺省值；调用方仍可追加说明。
    /// 此标志只描述本报告，不推断整帧行为，含义见 [`Self`]。
    pub fn skipped(stage: UiPipelineStage, dirty_reasons: Vec<UiPipelineDirtyReason>) -> Self {
        Self {
            stage,
            skipped: true,
            dirty_reasons,
            ..Self::default()
        }
    }
}
