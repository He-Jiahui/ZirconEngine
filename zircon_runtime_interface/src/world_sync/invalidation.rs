use serde::{Deserialize, Serialize};

use crate::resource::ResourceId;

use super::{EntityId, WatchToken};

/// Stable summary of the runtime-owned dynamic-scene reload apply report.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetReloadFrameApplyReportDto {
    pub applied: u64,
    pub failed: u64,
    pub stale: u64,
    pub pending_count: u64,
}

/// Runtime facts that can invalidate editor projections without carrying editor state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WorldFact {
    Spawned(EntityId),
    Despawned(EntityId),
    Reparented {
        entity: EntityId,
        new_parent: Option<EntityId>,
    },
    SceneLoaded {
        scene: ResourceId,
    },
    SceneUnloaded {
        scene: ResourceId,
    },
    WorldReplaced {
        replacement_epoch: u64,
    },
    AssetReloadApplied(AssetReloadFrameApplyReportDto),
}

/// One frame's monotonic generation plus the subscriptions and facts it invalidated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvalidationBatch {
    pub generation: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dirty: Vec<WatchToken>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub facts: Vec<WorldFact>,
}

impl InvalidationBatch {
    /// Returns true when runtime dirty tokens are strictly increasing and therefore unique.
    ///
    /// Runtime subscription tables emit this canonical form directly from their ordered dirty
    /// index. Consumers may use it as a no-allocation projection fast path; malformed transport
    /// input remains valid wire data and must stay observable through the diagnostic slow path.
    pub fn has_canonical_dirty_tokens(&self) -> bool {
        self.dirty.windows(2).all(|tokens| tokens[0] < tokens[1])
    }
}
