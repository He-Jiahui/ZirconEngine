use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphicsError {
    #[error("wgpu surface acquisition status: {0}")]
    SurfaceStatus(&'static str),
    #[error("surface creation failed: {0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),
    #[error("no compatible adapter found")]
    NoAdapter,
    #[error("render device request failed: {0}")]
    DeviceRequest(#[from] zr_rhi::RenderDeviceRequestFailure),
    #[error("render adapter selection failed: {0}")]
    AdapterSelection(#[from] zr_rhi::AdapterSelectionError),
    #[error("render device profile negotiation failed: {0}")]
    DeviceNegotiation(#[from] zr_rhi::RenderDeviceNegotiationError),
    #[error("neutral render hardware interface failed: {0}")]
    Rhi(#[from] zr_rhi::RhiError),
    #[error("render frame submission receipt failed: {0}")]
    FrameSubmissionReceipt(
        #[from] crate::core::framework::render::RenderFrameSubmissionReceiptError,
    ),
    #[error("render scene submission completion failed: {0}")]
    SceneSubmissionCompletion(
        #[from] crate::core::framework::render::RenderSceneSubmissionCompletionError,
    ),
    #[error("render frame submission failure receipt failed: {0}")]
    FrameSubmissionFailureReceipt(
        #[from] crate::core::framework::render::RenderFrameSubmissionFailureReceiptError,
    ),
    #[error("render frame failed after settling its submitted work: {source}")]
    FrameSubmissionFailed {
        receipt: crate::core::framework::render::RenderFrameSubmissionFailureReceipt,
        #[source]
        source: Box<GraphicsError>,
    },
    #[error("render frame failed and pre-scene submission settlement failed: {settlement}")]
    FrameSubmissionSettlement {
        settlement: String,
        #[source]
        source: Box<GraphicsError>,
    },
    #[error(
        "pre-scene producer submission {ticket:?} failed ledger registration and settled as {status:?}: {source}"
    )]
    FrameProducerRegistrationFailed {
        ticket: zr_rhi::SubmissionTicket,
        status: zr_rhi::SubmissionStatus,
        #[source]
        source: Box<GraphicsError>,
    },
    #[error("render frame presentation failed after scene submission: {source}")]
    FramePresentationFailed {
        receipt: crate::core::framework::render::RenderFrameSubmissionReceipt,
        present_submission: Option<zr_rhi::SubmissionTicket>,
        #[source]
        source: Box<GraphicsError>,
    },
    #[error("surface frame cleanup failed after presentation work failed: {cleanup}")]
    SurfaceFrameCleanupFailed {
        cleanup: String,
        #[source]
        source: Box<GraphicsError>,
    },
    #[error("render viewport product publication failed after scene submission: {source}")]
    FrameProductPublicationFailed {
        receipt: crate::core::framework::render::RenderFrameSubmissionReceipt,
        product_submission: Option<zr_rhi::SubmissionTicket>,
        #[source]
        source: Box<GraphicsError>,
    },
    #[error(
        "render scene submission {scene_submission:?} succeeded before frame finalization failed: {source}"
    )]
    FrameFailedAfterSceneSubmission {
        scene_submission: zr_rhi::SubmissionTicket,
        #[source]
        source: Box<GraphicsError>,
    },
    #[error(
        "submitted scene finalization failed (readback: {readback:?}, environment IBL: {environment_ibl:?})"
    )]
    SceneSubmissionFinalization {
        readback: Option<String>,
        environment_ibl: Option<String>,
    },
    #[error("wgpu validation failed: {0}")]
    WgpuValidation(String),
    #[error("asset channel failure: {0}")]
    Channel(String),
    #[error("asset loading failed: {0}")]
    Asset(String),
    #[error("runtime service resolution failed: {0}")]
    RuntimeService(String),
    #[error("thread bootstrap failure: {0}")]
    ThreadBootstrap(String),
    #[error("buffer map failed: {0}")]
    BufferMap(String),
    #[error("product diagnostic readback timed out after {timeout:?}")]
    DiagnosticReadbackTimedOut { timeout: std::time::Duration },
    #[error("graphics debugger capture failed: {0}")]
    GraphicsDebugger(String),
    #[error("advanced render provider selection failed: {0}")]
    AdvancedProviderSelection(String),
    #[error("offscreen frame target is unavailable after target initialization")]
    OffscreenTargetUnavailable,
    #[error("render view-family phase {phase:?} is unavailable during frame submission")]
    MissingViewFamilyPhase {
        phase: crate::core::framework::render::RenderPipelinePhase,
    },
    #[error("live frame graph resource `{resource}` has no physical backing")]
    MissingFrameGraphResourceBacking { resource: &'static str },
    #[error("compiled scene draws have no prepared GPU Scene upload")]
    MissingPreparedGpuSceneUpload,
    #[error("scene uniform upload range is invalid for `{label}`")]
    InvalidBufferUploadRange { label: &'static str },
    #[error(
        "scene renderer device epoch mismatch: expected {expected_device_id:?}/{expected_generation:?}, actual {actual_device_id:?}/{actual_generation:?}"
    )]
    SceneRendererDeviceEpochMismatch {
        expected_device_id: zr_rhi::DeviceId,
        expected_generation: zr_rhi::DeviceGeneration,
        actual_device_id: zr_rhi::DeviceId,
        actual_generation: zr_rhi::DeviceGeneration,
    },
    #[error(
        "runtime prepare GPU readback device epoch mismatch: expected {expected_device_id:?}/{expected_generation:?}, actual {actual_device_id:?}/{actual_generation:?}"
    )]
    RuntimePrepareDeviceEpochMismatch {
        expected_device_id: zr_rhi::DeviceId,
        expected_generation: zr_rhi::DeviceGeneration,
        actual_device_id: zr_rhi::DeviceId,
        actual_generation: zr_rhi::DeviceGeneration,
    },
}

impl GraphicsError {
    pub(crate) const fn submitted_scene_submission(&self) -> Option<zr_rhi::SubmissionTicket> {
        match self {
            Self::FrameFailedAfterSceneSubmission {
                scene_submission, ..
            } => Some(*scene_submission),
            _ => None,
        }
    }

    pub fn frame_submission_failure_receipt(
        &self,
    ) -> Option<&crate::core::framework::render::RenderFrameSubmissionFailureReceipt> {
        match self {
            Self::FrameSubmissionFailed { receipt, .. } => Some(receipt),
            _ => None,
        }
    }

    pub fn frame_submission_receipt(
        &self,
    ) -> Option<&crate::core::framework::render::RenderFrameSubmissionReceipt> {
        match self {
            Self::FramePresentationFailed { receipt, .. }
            | Self::FrameProductPublicationFailed { receipt, .. } => Some(receipt),
            _ => None,
        }
    }
}
