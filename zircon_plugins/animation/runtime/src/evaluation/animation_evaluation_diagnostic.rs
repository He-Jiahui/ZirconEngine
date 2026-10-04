//! 场景事件中的实体级评估失败记录；发布者应让订阅者能定位到失败实体及骨架、剪辑修订。
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
