use zircon_runtime::scene::EntityId;

use super::{AnimationAssetRevision, AnimationEvaluationError};

/// Deduplicated production evaluation failure emitted through the scene event store.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationEvaluationDiagnostic {
    pub entity: EntityId,
    pub skeleton: AnimationAssetRevision,
    pub clip: AnimationAssetRevision,
    pub error: AnimationEvaluationError,
}
