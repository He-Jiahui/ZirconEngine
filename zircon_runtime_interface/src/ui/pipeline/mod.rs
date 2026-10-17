//! 界面流水线报告的共享契约：阶段标识、失效原因、阶段观测与帧汇总。
//! Runtime 填充观测，Editor 读取快照；本模块不执行调度或重建。
//! 报告拥有向量与说明文本，跨帧共享由调用方管理。

#[path = "dirty_reason.rs"]
mod dirty_reason;
#[path = "frame_report.rs"]
mod frame_report;
#[path = "stage.rs"]
mod stage;
#[path = "stage_counters.rs"]
mod stage_counters;
#[path = "stage_report.rs"]
mod stage_report;

pub use dirty_reason::UiPipelineDirtyReason;
pub use frame_report::UiPipelineFrameReport;
pub use stage::UiPipelineStage;
pub use stage_counters::UiPipelineStageCounters;
pub use stage_report::UiPipelineStageReport;
