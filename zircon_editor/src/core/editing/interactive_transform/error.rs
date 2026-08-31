use thiserror::Error;
use zircon_runtime::scene::{NodeId, SceneError};
use zircon_runtime_interface::math::{AffineInverseError, NumericError};

use crate::core::editor_message::DocumentId;

#[derive(Debug, Error)]
pub(crate) enum InteractiveTransformError {
    #[error("interactive transform requires at least one selected entity")]
    EmptySelection,
    #[error("interactive transform primary entity {primary} is not selected")]
    PrimaryNotSelected { primary: NodeId },
    #[error("interactive transform document changed from {expected:?} to {actual:?}")]
    DocumentChanged {
        expected: DocumentId,
        actual: Option<DocumentId>,
    },
    #[error("interactive transform request targets {actual}, expected primary root {expected}")]
    PrimaryTargetMismatch { expected: NodeId, actual: NodeId },
    #[error("interactive transform target {entity} is missing a local or world transform")]
    TargetUnavailable { entity: NodeId },
    #[error("interactive transform target {entity} is not transform-mutable")]
    TargetNotMutable { entity: NodeId },
    #[error("interactive transform parent {parent} for entity {entity} has no world matrix")]
    ParentWorldUnavailable { entity: NodeId, parent: NodeId },
    #[error("interactive transform parent matrix for entity {entity} is not invertible")]
    ParentInverse {
        entity: NodeId,
        #[source]
        source: AffineInverseError,
    },
    #[error("interactive transform pivot matrix is not invertible")]
    PivotInverse {
        #[source]
        source: AffineInverseError,
    },
    #[error("interactive transform world generation changed from {expected} to {actual}")]
    StaleWorldGeneration { expected: u64, actual: u64 },
    #[error("interactive transform target for entity {entity} is invalid")]
    InvalidTransform {
        entity: NodeId,
        #[source]
        source: NumericError,
    },
    #[error(
        "interactive transform target for entity {entity} cannot be represented as TRS (residual {recomposition_residual})"
    )]
    NonRepresentableTransform {
        entity: NodeId,
        recomposition_residual: f32,
    },
    #[error("interactive transform could not update entity {entity}")]
    SceneMutation {
        entity: NodeId,
        #[source]
        source: SceneError,
    },
    #[error("interactive transform update failed and rollback also failed: {rollback}")]
    PreviewRollbackFailed {
        #[source]
        cause: Box<InteractiveTransformError>,
        rollback: String,
    },
    #[error(
        "interactive transform cancellation failed and preview restoration also failed: {rollback}"
    )]
    CancelRollbackFailed {
        #[source]
        cause: Box<InteractiveTransformError>,
        rollback: String,
    },
}
