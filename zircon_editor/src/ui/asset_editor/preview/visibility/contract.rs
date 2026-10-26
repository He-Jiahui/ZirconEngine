use std::sync::Arc;

use zircon_runtime::core::math::Real;
use zircon_runtime::ui::surface::UiPropertyMutationReport;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, surface::UiSurfaceFrame, tree::UiTreeError,
    v2::UiTemplateNodeInstancePathStep,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiPreviewVisibilitySource {
    pub document_uri: String,
    pub asset_id: String,
    pub source_revision: u64,
    pub last_valid_generation: u64,
}

/// The first probe accepts only a native Label directly under the owning view.
/// Empty ancestry is explicit; a missing or repeated instance is never guessed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiPreviewVisibilityTarget {
    pub source_path: String,
    pub root_node_id: String,
    pub source_node_id: String,
    pub instance_path: Vec<UiTemplateNodeInstancePathStep>,
}

#[derive(Clone, Debug)]
pub(crate) struct UiPreviewVisibilityGesture {
    pub(super) lease: Arc<()>,
    pub(super) source: UiPreviewVisibilitySource,
    pub(super) target: UiPreviewVisibilityTarget,
}

impl UiPreviewVisibilityGesture {
    pub(crate) fn source(&self) -> &UiPreviewVisibilitySource {
        &self.source
    }

    pub(crate) fn target(&self) -> &UiPreviewVisibilityTarget {
        &self.target
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum UiPreviewVisibilityError {
    #[error("visibility preview needs the current valid v2 source")]
    InvalidSource,
    #[error("visibility preview requires an unbound native view without imports or styles")]
    UnsupportedDocument,
    #[error("visibility preview requires a native Label directly under the root Panel")]
    UnsupportedTarget,
    #[error("the preview or its source metadata is unavailable")]
    PreviewUnavailable,
    #[error("the source target does not resolve to exactly one current node")]
    AmbiguousTarget,
    #[error("another visibility gesture is still active")]
    GestureInProgress,
    #[error("the gesture is not owned by this PreviewHost")]
    RetiredGesture,
    #[error("the authoring source changed since this gesture began")]
    StaleSource,
    #[error("the request does not carry the current gesture frame")]
    StaleFrame,
    #[error("the request sequence was already consumed or is older")]
    StaleRequest,
    #[error("another update replaced the gesture's expected frame")]
    FrameChanged,
    #[error("the target incarnation, state or override presence changed")]
    TargetChanged,
    #[error("the preview window is closed or destroyed")]
    WindowUnavailable,
    #[error("sampling requires a valid Boolean Step channel and a finite time in seconds")]
    UnsupportedChannelSample,
}

/// Publication is separate from property acceptance and never claims a present.
#[derive(Debug)]
pub(crate) enum UiPreviewVisibilityPublication {
    NotRequested,
    Published(Arc<UiSurfaceFrame>),
    Failed(UiTreeError),
    VerificationFailed(Arc<UiSurfaceFrame>),
}

#[derive(Debug)]
pub(crate) enum UiPreviewVisibilityOutcome {
    Rejected(UiPreviewVisibilityError),
    /// A Runtime error can occur after mutation; it is not a preflight rejection.
    RuntimeFailed(UiTreeError),
    Reported {
        report: UiPropertyMutationReport,
        publication: UiPreviewVisibilityPublication,
    },
}

/// A pure channel query followed by the existing property request, not a clock or present ACK.
#[derive(Debug)]
pub(crate) struct UiPreviewVisibilitySample {
    pub requested_seconds: Real,
    pub sampled_visible: bool,
    pub outcome: UiPreviewVisibilityOutcome,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum UiPreviewVisibilityResetError {
    #[error(transparent)]
    Rejected(#[from] UiPreviewVisibilityError),
    #[error(transparent)]
    Rebuild(#[from] crate::ui::asset_editor::UiAssetEditorSessionError),
}

#[derive(Debug)]
pub(in crate::ui::asset_editor::preview::preview_host) struct PreviewVisibilityState {
    pub(super) gesture: UiPreviewVisibilityGesture,
    pub(super) frame: Arc<UiSurfaceFrame>,
    pub(super) node: UiNodeId,
    pub(super) incarnation: u64,
    pub(super) root: UiNodeId,
    pub(super) root_incarnation: u64,
    pub(super) baseline_visible: bool,
    pub(super) baseline_attribute: Option<bool>,
    pub(super) current_visible: bool,
    pub(super) current_attribute: Option<bool>,
    pub(super) last_sequence: u64,
}

impl PartialEq for PreviewVisibilityState {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.gesture.lease, &other.gesture.lease)
            && Arc::ptr_eq(&self.frame, &other.frame)
            && self.gesture.source == other.gesture.source
            && self.gesture.target == other.gesture.target
            && self.node == other.node
            && self.incarnation == other.incarnation
            && self.root == other.root
            && self.root_incarnation == other.root_incarnation
            && self.baseline_visible == other.baseline_visible
            && self.baseline_attribute == other.baseline_attribute
            && self.current_visible == other.current_visible
            && self.current_attribute == other.current_attribute
            && self.last_sequence == other.last_sequence
    }
}
