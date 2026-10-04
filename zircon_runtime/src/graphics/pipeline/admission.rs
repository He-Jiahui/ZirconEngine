//! 绘制管线准入区分可等待的异步编译状态与本帧不可恢复的失败，调用方据此重试或报告。
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipelineAdmissionReason {
    SourceValidationQueued,
    SourceValidationPending,
    CompileQueued,
    CompilePending,
    QueueSaturated,
    CompilationDisabled,
    WorkerUnavailable,
    JobPanicked,
    UnknownVariant,
    WrongPass,
    GeometrySourceUnavailable,
    SourceAssemblyFailed,
    SourceValidationFailed,
    ShaderInterfaceMismatch,
    OitFragmentStoreUnavailable,
    PipelineValidationFailed,
}

impl PipelineAdmissionReason {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::SourceValidationQueued => "source_validation_queued",
            Self::SourceValidationPending => "source_validation_pending",
            Self::CompileQueued => "compile_queued",
            Self::CompilePending => "compile_pending",
            Self::QueueSaturated => "queue_saturated",
            Self::CompilationDisabled => "compilation_disabled",
            Self::WorkerUnavailable => "worker_unavailable",
            Self::JobPanicked => "job_panicked",
            Self::UnknownVariant => "unknown_variant",
            Self::WrongPass => "wrong_pass",
            Self::GeometrySourceUnavailable => "geometry_source_unavailable",
            Self::SourceAssemblyFailed => "source_assembly_failed",
            Self::SourceValidationFailed => "source_validation_failed",
            Self::ShaderInterfaceMismatch => "shader_interface_mismatch",
            Self::OitFragmentStoreUnavailable => "oit_fragment_store_unavailable",
            Self::PipelineValidationFailed => "pipeline_validation_failed",
        }
    }

    pub(crate) const fn is_terminal(self) -> bool {
        !matches!(
            self,
            Self::SourceValidationQueued
                | Self::SourceValidationPending
                | Self::CompileQueued
                | Self::CompilePending
                | Self::QueueSaturated
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PipelineUnavailable {
    reason: PipelineAdmissionReason,
    state_age: Duration,
}

impl PipelineUnavailable {
    pub(crate) const fn reason(self) -> PipelineAdmissionReason {
        self.reason
    }

    pub(crate) const fn state_age(self) -> Duration {
        self.state_age
    }
}

#[derive(Debug)]
#[must_use = "pipeline admission must be handled before recording a draw"]
pub(crate) enum PipelineAdmission<T> {
    Ready(T),
    Deferred(PipelineUnavailable),
    Failed(PipelineUnavailable),
}

impl<T> PipelineAdmission<T> {
    pub(crate) const fn unavailable(reason: PipelineAdmissionReason, state_age: Duration) -> Self {
        let unavailable = PipelineUnavailable { reason, state_age };
        if reason.is_terminal() {
            Self::Failed(unavailable)
        } else {
            Self::Deferred(unavailable)
        }
    }

    pub(crate) const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    pub(crate) const fn is_deferred(&self) -> bool {
        matches!(self, Self::Deferred(_))
    }

    pub(crate) const fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    pub(crate) const fn unavailable_details(&self) -> Option<PipelineUnavailable> {
        match self {
            Self::Ready(_) => None,
            Self::Deferred(unavailable) | Self::Failed(unavailable) => Some(*unavailable),
        }
    }
}

#[cfg(test)]
#[path = "tests/admission.rs"]
mod tests;
