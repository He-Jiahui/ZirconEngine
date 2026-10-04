use std::sync::Arc;

use serde::{Deserialize, Serialize};
use zircon_runtime::scene::EntityId;

use super::{
    SceneInspectionFieldsDelta, SceneInspectionHierarchyAnchor, SceneInspectionSelectionDelta,
};

/// Runtime-scene change notification without a copied hierarchy or inspector snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionMessage {
    previous_generation: Option<u64>,
    generation: u64,
    focused_entity: Option<EntityId>,
    added_anchors: Arc<[SceneInspectionHierarchyAnchor]>,
    changed_anchors: Arc<[SceneInspectionHierarchyAnchor]>,
    removed_entities: Arc<[EntityId]>,
    hierarchy_reflow_required: bool,
    focused_fields: SceneInspectionFieldsDelta,
    selection: SceneInspectionSelectionDelta,
}

impl SceneInspectionMessage {
    /// 供持有上一层级代次的消费者应用稀疏变化；树结构与字段值仍从运行时检查产物读取。
    pub fn delta(
        previous_generation: u64,
        generation: u64,
        focused_entity: Option<EntityId>,
        added_anchors: Vec<SceneInspectionHierarchyAnchor>,
        changed_anchors: Vec<SceneInspectionHierarchyAnchor>,
        removed_entities: Vec<EntityId>,
        hierarchy_reflow_required: bool,
        focused_fields: SceneInspectionFieldsDelta,
        selection: SceneInspectionSelectionDelta,
    ) -> Self {
        Self {
            previous_generation: Some(previous_generation),
            generation,
            focused_entity,
            added_anchors: added_anchors.into(),
            changed_anchors: changed_anchors.into(),
            removed_entities: removed_entities.into(),
            hierarchy_reflow_required,
            focused_fields,
            selection,
        }
    }

    /// The receiver has no compatible base generation and must read the runtime artifact anew.
    pub fn resync(generation: u64, focused_entity: Option<EntityId>) -> Self {
        Self::resync_with_selection_revision(generation, focused_entity, 0)
    }

    pub fn resync_with_selection_revision(
        generation: u64,
        focused_entity: Option<EntityId>,
        selection_revision: u64,
    ) -> Self {
        Self {
            previous_generation: None,
            generation,
            focused_entity,
            added_anchors: Vec::new().into(),
            changed_anchors: Vec::new().into(),
            removed_entities: Vec::new().into(),
            hierarchy_reflow_required: true,
            focused_fields: SceneInspectionFieldsDelta::resync(focused_entity),
            selection: SceneInspectionSelectionDelta::resync_at(selection_revision),
        }
    }

    // 最新消息只合并选择修订链，不拼接层级或字段差；消费方按 previous_generation 验证基线并在缺口时重同步。
    pub(in crate::core::editor_message) fn coalesce_selection_from(&mut self, previous: &Self) {
        self.selection.coalesce_from(&previous.selection);
    }

    pub(crate) fn with_selection_resync_at(mut self, selection_revision: u64) -> Self {
        self.selection = SceneInspectionSelectionDelta::resync_at(selection_revision);
        self
    }

    pub const fn previous_generation(&self) -> Option<u64> {
        self.previous_generation
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn focused_entity(&self) -> Option<EntityId> {
        self.focused_entity
    }

    pub fn added_anchors(&self) -> &[SceneInspectionHierarchyAnchor] {
        &self.added_anchors
    }

    pub fn changed_anchors(&self) -> &[SceneInspectionHierarchyAnchor] {
        &self.changed_anchors
    }

    pub fn removed_entities(&self) -> &[EntityId] {
        &self.removed_entities
    }

    /// The producer rebuilt the hierarchy, so this message cannot be applied as a sparse patch.
    pub const fn requires_hierarchy_reflow(&self) -> bool {
        self.hierarchy_reflow_required
    }

    pub fn focused_fields(&self) -> &SceneInspectionFieldsDelta {
        &self.focused_fields
    }

    pub fn selection(&self) -> &SceneInspectionSelectionDelta {
        &self.selection
    }

    pub const fn requires_resync(&self) -> bool {
        self.previous_generation.is_none()
    }
}

#[cfg(test)]
#[path = "tests/message.rs"]
mod tests;
