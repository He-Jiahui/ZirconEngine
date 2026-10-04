use crate::asset::artifact::{
    RenderArtifactBlockIoDispatchBudget, RenderArtifactBlockIoDispatchError,
    RenderArtifactBlockIoDispatchReport, RenderArtifactBlockLoaderCloseReport,
    RenderArtifactManifestIoDispatchBudget, RenderArtifactManifestIoDispatchError,
    RenderArtifactManifestIoDispatchReport, RenderArtifactManifestLoaderCloseReport,
};
use crate::core::runtime::TaskGraphAdmissionError;

use super::super::{
    RenderAssetGpuUploadLimits, RenderAssetGpuUploadPlanError, RenderAssetResidencyRoute,
    RenderAssetResidencyTicket, RenderAssetResidencyTicketId, RenderAssetResidencyTransitionError,
    RenderAssetSemanticLoadError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderAssetSemanticExecutorLimits {
    max_in_flight: usize,
    upload: RenderAssetGpuUploadLimits,
}

impl RenderAssetSemanticExecutorLimits {
    pub(crate) const fn new(max_in_flight: usize, upload: RenderAssetGpuUploadLimits) -> Self {
        Self {
            max_in_flight,
            upload,
        }
    }

    pub(crate) const fn max_in_flight(self) -> usize {
        self.max_in_flight
    }

    pub(crate) const fn upload(self) -> RenderAssetGpuUploadLimits {
        self.upload
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderAssetSemanticExecutorMaintenanceBudget {
    pub(super) manifest_io: RenderArtifactManifestIoDispatchBudget,
    pub(super) block_io: RenderArtifactBlockIoDispatchBudget,
    pub(super) max_load_advances: usize,
    pub(super) max_prepare_submissions: usize,
    pub(super) max_prepare_completions: usize,
}

impl RenderAssetSemanticExecutorMaintenanceBudget {
    pub(crate) const fn new(
        manifest_io: RenderArtifactManifestIoDispatchBudget,
        block_io: RenderArtifactBlockIoDispatchBudget,
        max_load_advances: usize,
        max_prepare_submissions: usize,
        max_prepare_completions: usize,
    ) -> Self {
        Self {
            manifest_io,
            block_io,
            max_load_advances,
            max_prepare_submissions,
            max_prepare_completions,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderAssetSemanticExecutorDiagnostics {
    pub(super) active_loads: usize,
    pub(super) ready_cpu: usize,
    pub(super) preparing_uploads: usize,
    pub(super) ready_uploads: usize,
    pub(super) failures: usize,
}

impl RenderAssetSemanticExecutorDiagnostics {
    pub(crate) const fn active_loads(self) -> usize {
        self.active_loads
    }

    pub(crate) const fn ready_cpu(self) -> usize {
        self.ready_cpu
    }

    pub(crate) const fn preparing_uploads(self) -> usize {
        self.preparing_uploads
    }

    pub(crate) const fn ready_uploads(self) -> usize {
        self.ready_uploads
    }

    pub(crate) const fn failures(self) -> usize {
        self.failures
    }

    pub(crate) const fn in_flight(self) -> usize {
        self.active_loads
            .saturating_add(self.ready_cpu)
            .saturating_add(self.preparing_uploads)
            .saturating_add(self.ready_uploads)
            .saturating_add(self.failures)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderAssetSemanticExecutorMaintenanceReport {
    pub(super) manifest_io: RenderArtifactManifestIoDispatchReport,
    pub(super) block_io: RenderArtifactBlockIoDispatchReport,
    pub(super) load_advances: usize,
    pub(super) deferred_loads: usize,
    pub(super) ready_cpu: usize,
    pub(super) prepare_submissions: usize,
    pub(super) prepare_completions: usize,
    pub(super) ready_uploads: usize,
    pub(super) failed: usize,
    pub(super) stale_discards: usize,
}

impl RenderAssetSemanticExecutorMaintenanceReport {
    pub(crate) const fn manifest_io(self) -> RenderArtifactManifestIoDispatchReport {
        self.manifest_io
    }

    pub(crate) const fn block_io(self) -> RenderArtifactBlockIoDispatchReport {
        self.block_io
    }

    pub(crate) const fn load_advances(self) -> usize {
        self.load_advances
    }

    pub(crate) const fn deferred_loads(self) -> usize {
        self.deferred_loads
    }

    pub(crate) const fn ready_cpu(self) -> usize {
        self.ready_cpu
    }

    pub(crate) const fn prepare_submissions(self) -> usize {
        self.prepare_submissions
    }

    pub(crate) const fn prepare_completions(self) -> usize {
        self.prepare_completions
    }

    pub(crate) const fn ready_uploads(self) -> usize {
        self.ready_uploads
    }

    pub(crate) const fn failed(self) -> usize {
        self.failed
    }

    pub(crate) const fn stale_discards(self) -> usize {
        self.stale_discards
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderAssetSemanticExecutorInitError {
    #[error("render semantic executor limit `{limit}` must be non-zero")]
    ZeroLimit { limit: &'static str },
    #[error("render semantic executor target platform must not be empty")]
    EmptyTargetPlatform,
    #[error(transparent)]
    Execution(#[from] TaskGraphAdmissionError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderAssetSemanticExecutorAdmissionError {
    #[error("render semantic executor is closed")]
    Closed,
    #[error("render semantic executor does not own residency route {actual:?}")]
    UnsupportedRoute { actual: RenderAssetResidencyRoute },
    #[error("render semantic executor already tracks ticket {ticket:?}")]
    DuplicateTicket {
        ticket: RenderAssetResidencyTicketId,
    },
    #[error("render semantic executor capacity {capacity} is full")]
    CapacityExceeded { capacity: usize },
    #[error(transparent)]
    Semantic(#[from] RenderAssetSemanticLoadError),
    #[error("render semantic executor residency transition failed: {0:?}")]
    Residency(RenderAssetResidencyTransitionError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderAssetSemanticExecutorMaintenanceError {
    #[error("render semantic executor is closed")]
    Closed,
    #[error(transparent)]
    ManifestIo(#[from] RenderArtifactManifestIoDispatchError),
    #[error(transparent)]
    BlockIo(#[from] RenderArtifactBlockIoDispatchError),
    #[error("render semantic executor prepare task identifier space is exhausted")]
    PrepareTaskIdExhausted,
    #[error(transparent)]
    PrepareExecution(#[from] TaskGraphAdmissionError),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RenderAssetSemanticExecutorCloseReport {
    pub(super) tickets: Vec<RenderAssetResidencyTicket>,
    pub(super) manifest_loader: RenderArtifactManifestLoaderCloseReport,
    pub(super) block_loader: RenderArtifactBlockLoaderCloseReport,
}

impl RenderAssetSemanticExecutorCloseReport {
    pub(crate) fn tickets(&self) -> &[RenderAssetResidencyTicket] {
        &self.tickets
    }

    pub(crate) const fn manifest_loader(&self) -> RenderArtifactManifestLoaderCloseReport {
        self.manifest_loader
    }

    pub(crate) const fn block_loader(&self) -> RenderArtifactBlockLoaderCloseReport {
        self.block_loader
    }
}

#[derive(Debug)]
pub(crate) enum RenderAssetSemanticExecutorWorkError {
    Semantic(RenderAssetSemanticLoadError),
    UploadPlan(RenderAssetGpuUploadPlanError),
    Residency(RenderAssetResidencyTransitionError),
}

#[derive(Debug)]
pub(crate) struct RenderAssetSemanticExecutorFailure {
    pub(super) ticket: RenderAssetResidencyTicket,
    pub(super) error: RenderAssetSemanticExecutorWorkError,
    pub(super) terminal_transition_error: Option<RenderAssetResidencyTransitionError>,
}

impl RenderAssetSemanticExecutorFailure {
    pub(crate) fn ticket(&self) -> RenderAssetResidencyTicket {
        self.ticket.clone()
    }

    pub(crate) const fn error(&self) -> &RenderAssetSemanticExecutorWorkError {
        &self.error
    }

    pub(crate) const fn terminal_transition_error(
        &self,
    ) -> Option<RenderAssetResidencyTransitionError> {
        self.terminal_transition_error
    }
}
