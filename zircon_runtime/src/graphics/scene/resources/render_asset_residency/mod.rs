mod contract;
mod gpu_maintenance;
mod gpu_residency;
mod gpu_upload;
mod manager;
mod semantic_blocks;
mod semantic_executor;
mod work_queue;

pub(crate) use contract::{
    RenderAssetDemandGeneration, RenderAssetDeviceEpoch, RenderAssetResidencyAdmissionError,
    RenderAssetResidencyMutation, RenderAssetResidencyMutationStats, RenderAssetResidencyRelease,
    RenderAssetResidencyReleaseKind, RenderAssetResidencyRoute, RenderAssetResidencyScope,
    RenderAssetResidencyState, RenderAssetResidencyTicket, RenderAssetResidencyTicketId,
    RenderAssetResidencyTransitionError,
};
pub(crate) use gpu_maintenance::{
    RenderAssetGpuMaintenanceBudget, RenderAssetGpuMaintenanceFailure,
    RenderAssetGpuMaintenanceReport, RenderAssetGpuPollReceiptError,
};
pub(crate) use gpu_residency::{RenderAssetGpuResidencyLimits, RenderAssetGpuUploadBindFailure};
pub(crate) use gpu_upload::{
    RenderAssetGpuArtifact, RenderAssetGpuArtifactKind, RenderAssetGpuMeshArtifact,
    RenderAssetGpuMeshLod, RenderAssetGpuTextureArtifact, RenderAssetGpuUploadBudgetClass,
    RenderAssetGpuUploadLease, RenderAssetGpuUploadLimits, RenderAssetGpuUploadPlan,
    RenderAssetGpuUploadPlanError, RenderAssetGpuUploadPlanKind, RenderAssetGpuUploadQuote,
    RenderAssetGpuUploadSubmitError,
};
pub(crate) use manager::device_recovery::{
    RenderAssetDeviceRecoveryError, RenderAssetDeviceRecoveryReport,
};
pub(crate) use manager::RenderAssetResidencyManager;
pub(crate) use semantic_blocks::{
    RenderAssetCpuArtifactLease, RenderAssetCpuBlockLease, RenderAssetSemanticBlockLoad,
    RenderAssetSemanticBlockLoadAdvance, RenderAssetSemanticBlockLoadError,
    RenderAssetSemanticLoad, RenderAssetSemanticLoadAdvance, RenderAssetSemanticLoadError,
    RenderAssetSemanticLoadStage,
};
pub(crate) use semantic_executor::{
    RenderAssetSemanticExecutor, RenderAssetSemanticExecutorAdmissionError,
    RenderAssetSemanticExecutorCloseReport, RenderAssetSemanticExecutorDiagnostics,
    RenderAssetSemanticExecutorFailure, RenderAssetSemanticExecutorInitError,
    RenderAssetSemanticExecutorLimits, RenderAssetSemanticExecutorMaintenanceBudget,
    RenderAssetSemanticExecutorMaintenanceError, RenderAssetSemanticExecutorMaintenanceReport,
    RenderAssetSemanticExecutorOwner, RenderAssetSemanticExecutorOwnerAdmissionError,
    RenderAssetSemanticExecutorOwnerConfig, RenderAssetSemanticExecutorOwnerInitError,
    RenderAssetSemanticExecutorOwnerMaintenanceError, RenderAssetSemanticExecutorWorkError,
};
pub(crate) use work_queue::{
    RenderAssetResidencyWorkQueue, RenderAssetResidencyWorkQueueAdmissionFailure,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
