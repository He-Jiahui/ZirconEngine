mod capabilities;
mod presentation;
mod session;

pub(crate) use crate::core::editing::animation_document::{
    AnimationAuthoringDocumentKind as AnimationEditorDocumentKind, AnimationGraphNodeKind,
};
pub(crate) use capabilities::resolve_animation_graph_node_kind;
pub use capabilities::{
    animation_editor_capability_table, AnimationEditorCapabilityDescriptor,
    AnimationEditorCommandDiagnostic, AnimationEditorCommandRejectionReason,
};
pub use presentation::AnimationEditorPanePresentation;
pub use session::{
    AnimationCurveFoundationView, AnimationEditorBinaryKindMismatch, AnimationEditorSession,
    AnimationEditorSessionError, AnimationTimelineFoundationView,
};
