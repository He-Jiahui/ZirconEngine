//! 渲染块加载边界：先登记票据，再由调用方按预算派发 IO、维护截止时间并轮询结果。
//! 解码条目可跨多个请求共享，设备上传由 graphics 的语义驻留层负责。

#[path = "admission.rs"]
mod admission;
#[path = "contract.rs"]
mod contract;
#[path = "decode.rs"]
mod decode;
#[path = "dispatch.rs"]
mod dispatch;
#[path = "entry.rs"]
mod entry;
#[path = "loader.rs"]
mod loader;
#[path = "plan_batch.rs"]
mod plan_batch;
#[path = "policy.rs"]
mod policy;
#[path = "registry.rs"]
mod registry;
#[path = "worker.rs"]
mod worker;

pub use super::RenderArtifactIoPriority;
pub use contract::{
    RenderArtifactBlockAdmissionError, RenderArtifactBlockCancelReason, RenderArtifactBlockFailure,
    RenderArtifactBlockFailureCode, RenderArtifactBlockIoDispatchBudget,
    RenderArtifactBlockIoDispatchError, RenderArtifactBlockIoDispatchReport,
    RenderArtifactBlockLoadStage, RenderArtifactBlockLoaderCloseReport,
    RenderArtifactBlockLoaderDiagnostics, RenderArtifactBlockLoaderInitError,
    RenderArtifactBlockLoaderLimits, RenderArtifactBlockMaintenanceReport, RenderArtifactBlockPoll,
    RenderArtifactBlockRequest, RenderArtifactDecodedBlock,
};
pub use loader::{
    RenderArtifactBlockLoader, RenderArtifactBlockTicket, RenderArtifactBlockTicketBatch,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
