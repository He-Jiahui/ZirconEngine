#[path = "backend/mod.rs"]
mod backend;
#[path = "controller.rs"]
mod controller;
#[path = "edit_policy/mod.rs"]
mod edit_policy;
#[path = "edit_protection.rs"]
mod edit_protection;
#[path = "embedded_backend/mod.rs"]
mod embedded_backend;
#[path = "error.rs"]
mod error;
#[path = "live_link.rs"]
mod live_link;
#[path = "mode.rs"]
mod mode;
#[path = "pending_edits/mod.rs"]
mod pending_edits;
#[path = "plugin_activation/mod.rs"]
mod plugin_activation;
#[path = "preview_frame.rs"]
mod preview_frame;
#[path = "preview_input.rs"]
mod preview_input;
#[path = "process_backend/mod.rs"]
mod process_backend;
#[path = "request.rs"]
mod request;
#[path = "simulate_camera.rs"]
mod simulate_camera;
#[path = "snapshot/mod.rs"]
mod snapshot;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
#[path = "transition_report.rs"]
mod transition_report;

#[cfg(test)]
pub(crate) use backend::TestAttachablePlayBackend;
pub use backend::{
    NoopPlayBackend, PlayBackend, PlayBackendPoll, PlayBackendRetireReport,
    PlayBackendStartFailure, PlayBackendStartReport, PlayBackendStopReport, SharedPlayBackend,
};
pub use controller::PlaySessionController;
pub use edit_policy::{PlayEditDecision, PlayEditPolicy};
pub use edit_protection::{
    PlayEditBeginError, PlayEditProtection, PlayEditResolutionError, PlayEditRoute,
    PlayEditRouteError,
};
pub use embedded_backend::{
    EmbeddedPlayBackend, PlaySessionFactory, PlaySessionLaunchRequest, PlaySessionLease,
    PlaySessionRetireReport, SharedPlaySessionFactory,
};
pub use error::PlaySessionError;
pub use live_link::{
    PlayDomainLink, PlayDomainLinkError, PlayInstanceId, PlayTerminalGatewayDetachError,
    WorldDomain,
};
pub use mode::{PlayCleanupFailure, PlayKind, PlayMode, PlayModeKind};
pub use pending_edits::{
    PendingEditApplyBudget, PendingEditApplyFailure, PendingEditApplyReport,
    PendingEditDecisionPrompt, PendingEditDiscardReport, PendingEditEnqueueReport,
    PendingEditExitDecision, PendingEditId, PendingEditIntent, PendingEditPage,
    PendingEditPageCursor, PendingEditPageEntry, PendingEditQueue, PendingEditQueueError,
    PendingEditQueueLimits, PendingEditQueueSummary,
};
pub use plugin_activation::{
    NativePluginAdmissionResolver, NativePluginArtifactAuthorityResolver,
    NativePluginBridgeActivation, NoopPluginBridgeActivation, PluginBridgeActivation,
    PluginBridgeActivationReport, SharedPluginBridgeActivation,
};
pub use preview_frame::{PlayPreviewCaptureError, PlayPreviewFrame, PlayPreviewFrameIdentity};
pub use preview_input::PlayPreviewInputError;
pub use process_backend::{ProcessPlayBackend, ProcessPlayBackendInstallError};
pub use request::PlayStartRequest;
pub use simulate_camera::PlaySimulateCameraError;
pub use snapshot::{
    MaterializedPlayScene, PlaySceneSource, PlaySceneSourceError,
    PlaySnapshotMaterializationFailure, PlaySnapshotStore,
};
pub use transition_report::{PlayTransitionCause, PlayTransitionReport};
