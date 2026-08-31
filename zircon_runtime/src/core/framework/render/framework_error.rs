use thiserror::Error;
use zr_rhi::{SubmissionStatus, SubmissionTicket};

use super::{
    EnvironmentRuntimeSnapshotError, RenderCapabilityMismatchDetail, RenderFrameSubmissionReceipt,
    RenderPipelinePhase, RenderSceneSubmissionCompletionError,
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RenderFrameworkError {
    #[error("render framework viewport `{viewport}` does not exist")]
    UnknownViewport { viewport: u64 },
    #[error(
        "render framework viewport `{viewport}` changed during submit: expected generation {expected_generation}, found {actual_generation}"
    )]
    ViewportChanged {
        viewport: u64,
        expected_generation: u64,
        actual_generation: u64,
    },
    #[error("render framework pipeline `{pipeline}` does not exist")]
    UnknownPipeline { pipeline: u64 },
    #[error("render framework pipeline `{pipeline}` failed graph validation: {message}")]
    GraphCompileFailure { pipeline: u64, message: String },
    #[error(
        "render framework pipeline `{pipeline}` is not compatible with backend capabilities: {reason}"
    )]
    CapabilityMismatch {
        pipeline: u64,
        reason: String,
        missing: Vec<RenderCapabilityMismatchDetail>,
    },
    #[error("render framework capability `{capability}` is unsupported")]
    UnsupportedCapability { capability: String },
    #[error("render frame submission state is invalid: {state}")]
    InvalidSubmissionState { state: &'static str },
    #[error("render view-family phase {phase:?} is unavailable during frame submission")]
    MissingViewFamilyPhase { phase: RenderPipelinePhase },
    #[error("live frame graph resource `{resource}` has no physical backing")]
    MissingFrameGraphResourceBacking { resource: &'static str },
    #[error("compiled scene draws have no prepared GPU Scene upload")]
    MissingPreparedGpuSceneUpload,
    #[error("scene uniform upload range is invalid for `{label}`")]
    InvalidBufferUploadRange { label: &'static str },
    #[error("render viewport-pick request is invalid")]
    InvalidViewportPickRequest,
    #[error("render viewport-pick ticket `{ticket}` is invalid")]
    InvalidViewportPickTicket { ticket: u64 },
    #[error("render viewport-pick ticket `{ticket}` does not exist")]
    UnknownViewportPickTicket { ticket: u64 },
    #[error("render viewport-pick ticket capacity {limit} is exhausted")]
    ViewportPickCapacityExceeded { limit: usize },
    #[error("render viewport-pick ticket id space is exhausted")]
    ViewportPickTicketSpaceExhausted,
    #[error("render environment-capture handle `{handle}` does not exist")]
    UnknownEnvironmentCaptureHandle { handle: u64 },
    #[error("render environment-capture queue capacity {limit} is exhausted")]
    EnvironmentCaptureQueueCapacityExceeded { limit: usize },
    #[error("render environment-capture handle id space is exhausted")]
    EnvironmentCaptureHandleSpaceExhausted,
    #[error("render environment-capture persistence result capacity {limit} is exhausted")]
    EnvironmentCapturePersistenceResultCapacityExceeded { limit: usize },
    #[error(
        "render environment-capture `{capture_id}` generation {requested_generation} is not newer than live generation {live_generation}"
    )]
    EnvironmentCaptureGenerationNotNewer {
        capture_id: String,
        requested_generation: u64,
        live_generation: u64,
    },
    #[error("render scene submission completion failed: {0}")]
    SceneSubmissionCompletion(#[from] RenderSceneSubmissionCompletionError),
    #[error("render environment runtime snapshot is inconsistent: {0}")]
    EnvironmentRuntimeSnapshot(#[from] EnvironmentRuntimeSnapshotError),
    #[error("render backend error: {0}")]
    Backend(String),
    #[error("render viewport product publication failed after scene submission: {reason}")]
    FrameProductPublicationFailed {
        receipt: RenderFrameSubmissionReceipt,
        product_submission: Option<SubmissionTicket>,
        reason: String,
    },
    #[error(
        "pre-scene producer submission {ticket:?} failed ledger registration and settled as {status:?}: {reason}"
    )]
    FrameProducerRegistrationFailed {
        ticket: SubmissionTicket,
        status: SubmissionStatus,
        reason: String,
    },
}

impl RenderFrameworkError {
    pub fn frame_submission_receipt(&self) -> Option<&RenderFrameSubmissionReceipt> {
        match self {
            Self::FrameProductPublicationFailed { receipt, .. } => Some(receipt),
            _ => None,
        }
    }

    pub const fn product_submission(&self) -> Option<SubmissionTicket> {
        match self {
            Self::FrameProductPublicationFailed {
                product_submission, ..
            } => *product_submission,
            _ => None,
        }
    }
}
