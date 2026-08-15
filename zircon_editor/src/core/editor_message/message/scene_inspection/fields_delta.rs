use serde::{Deserialize, Serialize};
use zircon_runtime::scene::EntityId;

use super::SceneInspectionPropertyPath;

/// Focused-inspector change identities; field values stay in the runtime artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionFieldsDelta {
    entity: Option<EntityId>,
    requires_resync: bool,
    changed_properties: Vec<SceneInspectionPropertyPath>,
    removed_properties: Vec<SceneInspectionPropertyPath>,
}

impl SceneInspectionFieldsDelta {
    pub fn unchanged(entity: Option<EntityId>) -> Self {
        Self {
            entity,
            requires_resync: false,
            changed_properties: Vec::new(),
            removed_properties: Vec::new(),
        }
    }

    pub fn delta(
        entity: EntityId,
        changed_properties: Vec<SceneInspectionPropertyPath>,
        removed_properties: Vec<SceneInspectionPropertyPath>,
    ) -> Self {
        Self {
            entity: Some(entity),
            requires_resync: false,
            changed_properties,
            removed_properties,
        }
    }

    /// Selection changed or the consumer fell behind, so it must read the focused artifact again.
    pub fn resync(entity: Option<EntityId>) -> Self {
        Self {
            entity,
            requires_resync: true,
            changed_properties: Vec::new(),
            removed_properties: Vec::new(),
        }
    }

    pub const fn entity(&self) -> Option<EntityId> {
        self.entity
    }

    pub const fn requires_resync(&self) -> bool {
        self.requires_resync
    }

    pub fn changed_properties(&self) -> &[SceneInspectionPropertyPath] {
        &self.changed_properties
    }

    pub fn removed_properties(&self) -> &[SceneInspectionPropertyPath] {
        &self.removed_properties
    }
}
