use std::sync::Arc;

use serde::{Deserialize, Serialize};
use zircon_runtime::scene::EntityId;

use super::SceneInspectionPropertyPath;

/// Focused-inspector change identities; field values stay in the runtime artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionFieldsDelta {
    entity: Option<EntityId>,
    requires_resync: bool,
    changed_properties: Arc<[SceneInspectionPropertyPath]>,
    removed_properties: Arc<[SceneInspectionPropertyPath]>,
}

impl SceneInspectionFieldsDelta {
    pub fn unchanged(entity: Option<EntityId>) -> Self {
        Self {
            entity,
            requires_resync: false,
            changed_properties: Vec::new().into(),
            removed_properties: Vec::new().into(),
        }
    }

    /// 只通报同一焦点对象的字段身份变化；切换焦点或无法建立前后状态时，发布者应使用 resync。
    pub fn delta(
        entity: EntityId,
        changed_properties: Vec<SceneInspectionPropertyPath>,
        removed_properties: Vec<SceneInspectionPropertyPath>,
    ) -> Self {
        Self {
            entity: Some(entity),
            requires_resync: false,
            changed_properties: changed_properties.into(),
            removed_properties: removed_properties.into(),
        }
    }

    /// Selection changed or the consumer fell behind, so it must read the focused artifact again.
    pub fn resync(entity: Option<EntityId>) -> Self {
        Self {
            entity,
            requires_resync: true,
            changed_properties: Vec::new().into(),
            removed_properties: Vec::new().into(),
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

#[cfg(test)]
#[path = "tests/fields_delta.rs"]
mod tests;
