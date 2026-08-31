use serde::{Deserialize, Serialize};

use super::SceneSystemTickPolicy;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InternalSceneSystem {
    ApplyDeferred,
    UpdateEvents,
    HierarchyValidity,
    ActiveHierarchy,
    WorldTransform,
    NodeCache,
    RenderExtractPrepare,
}

impl InternalSceneSystem {
    /// Declares the clock and pause contract for built-in maintenance.
    ///
    /// Event retention advances once per outer frame, including while virtual
    /// simulation is paused. All other built-ins derive or mutate game state
    /// and therefore remain on the virtual-time path.
    pub(crate) const fn tick_policy(self) -> SceneSystemTickPolicy {
        match self {
            Self::UpdateEvents => SceneSystemTickPolicy::monotonic_real(),
            Self::ApplyDeferred
            | Self::HierarchyValidity
            | Self::ActiveHierarchy
            | Self::WorldTransform
            | Self::NodeCache
            | Self::RenderExtractPrepare => SceneSystemTickPolicy::virtual_time(),
        }
    }
}
