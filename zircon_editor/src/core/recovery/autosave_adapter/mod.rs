//! 后台自动保存的适配层入口；计划选择、快照捕获、作业写入与结果归档在类型上分离。

#[path = "adapter.rs"]
mod adapter;
#[path = "model.rs"]
mod model;
#[path = "write_job.rs"]
mod write_job;

pub use adapter::AutosaveJobAdapter;
pub use model::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDocumentOutcome,
    AutosaveDocumentOutcomeKind, AutosaveDocumentRequest, AutosaveFailureStage,
    AutosaveHealthTelemetry, AutosaveRetryability, AutosaveSnapshot, AutosaveSnapshotSource,
    AutosaveWriteResult, DEFAULT_AUTOSAVE_COMPLETION_BUDGET,
};
