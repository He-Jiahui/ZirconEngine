//! 渲染清单加载边界：按资源修订与平台合并请求，显式派发后由票据轮询验证结果。
//! 它先于 block loader 运行，使驻留层能依据清单选择需要的子资源。

mod admission;
mod contract;
mod dispatch;
mod loader;
mod state;
mod worker;

pub use contract::{
    RenderArtifactManifestAdmissionError, RenderArtifactManifestCancelReason,
    RenderArtifactManifestFailure, RenderArtifactManifestFailureCode,
    RenderArtifactManifestIoDispatchBudget, RenderArtifactManifestIoDispatchError,
    RenderArtifactManifestIoDispatchReport, RenderArtifactManifestLoadStage,
    RenderArtifactManifestLoaderCloseReport, RenderArtifactManifestLoaderDiagnostics,
    RenderArtifactManifestLoaderInitError, RenderArtifactManifestLoaderLimits,
    RenderArtifactManifestMaintenanceReport, RenderArtifactManifestPoll,
    RenderArtifactManifestRequest, RenderArtifactManifestRequestKey,
};
pub use loader::{
    RenderArtifactManifestLoader, RenderArtifactManifestTicket, RenderArtifactManifestTicketBatch,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
