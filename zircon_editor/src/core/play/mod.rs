mod backend;
mod controller;
mod edit_policy;
mod edit_protection;
mod embedded_backend;
mod error;
mod live_link;
mod mode;
mod pending_edits;
mod plugin_activation;
mod preview_frame;
mod preview_input;
mod process_backend;
mod request;
mod simulate_camera;
mod snapshot;
#[cfg(test)]
mod tests;
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
