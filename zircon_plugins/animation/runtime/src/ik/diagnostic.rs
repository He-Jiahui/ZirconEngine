use zircon_runtime::asset::AssetId;
use zircon_runtime::scene::EntityId;

use super::AnimationIkExecutionError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationIkDiagnostic {
    pub entity: EntityId,
    pub skeleton: Option<AssetId>,
    pub error: AnimationIkExecutionError,
}
