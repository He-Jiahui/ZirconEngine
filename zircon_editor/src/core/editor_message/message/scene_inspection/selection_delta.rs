use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use zircon_runtime::scene::EntityId;

/// Revision-checked editor selection changes for a retained hierarchy projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionSelectionDelta {
    previous_revision: Option<u64>,
    revision: u64,
    added_entities: Vec<EntityId>,
    removed_entities: Vec<EntityId>,
}

impl SceneInspectionSelectionDelta {
    pub fn unchanged() -> Self {
        Self::unchanged_at(0)
    }

    pub fn unchanged_at(revision: u64) -> Self {
        Self {
            previous_revision: Some(revision),
            revision,
            added_entities: Vec::new(),
            removed_entities: Vec::new(),
        }
    }

    pub fn delta(added_entities: Vec<EntityId>, removed_entities: Vec<EntityId>) -> Self {
        Self::between(0, 1, added_entities, removed_entities)
    }

    /// 描述两个已知选择修订之间的集合差；消费者必须持有 previous_revision，缺口改走 resync。
    pub fn between(
        previous_revision: u64,
        revision: u64,
        added_entities: Vec<EntityId>,
        removed_entities: Vec<EntityId>,
    ) -> Self {
        Self {
            previous_revision: Some(previous_revision),
            revision,
            added_entities,
            removed_entities,
        }
    }

    /// The receiving projection has no compatible selection revision.
    pub fn resync() -> Self {
        Self::resync_at(0)
    }

    pub fn resync_at(revision: u64) -> Self {
        Self {
            previous_revision: None,
            revision,
            added_entities: Vec::new(),
            removed_entities: Vec::new(),
        }
    }

    /// Composes a superseded Latest delta into this newer delta.
    // 保留队列把连续选择差合成从最早保留修订到最新修订的差；链断开或任一端要求重同步时放弃差量。
    pub(super) fn coalesce_from(&mut self, previous: &Self) {
        if previous.requires_resync()
            || self.requires_resync()
            || self.previous_revision != Some(previous.revision)
        {
            *self = Self::resync_at(self.revision);
            return;
        }

        let mut added = HashSet::with_capacity(
            previous
                .added_entities
                .len()
                .saturating_add(self.added_entities.len()),
        );
        added.extend(previous.added_entities.iter().copied());
        let mut removed = HashSet::with_capacity(
            previous
                .removed_entities
                .len()
                .saturating_add(self.removed_entities.len()),
        );
        removed.extend(previous.removed_entities.iter().copied());
        for entity in &self.added_entities {
            if !removed.remove(entity) {
                added.insert(*entity);
            }
        }
        for entity in &self.removed_entities {
            if !added.remove(entity) {
                removed.insert(*entity);
            }
        }
        self.previous_revision = previous.previous_revision;
        self.added_entities = added.into_iter().collect();
        self.added_entities.sort_unstable();
        self.removed_entities = removed.into_iter().collect();
        self.removed_entities.sort_unstable();
    }

    pub const fn previous_revision(&self) -> Option<u64> {
        self.previous_revision
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub const fn requires_resync(&self) -> bool {
        self.previous_revision.is_none()
    }

    pub fn added_entities(&self) -> &[EntityId] {
        &self.added_entities
    }

    pub fn removed_entities(&self) -> &[EntityId] {
        &self.removed_entities
    }
}

#[cfg(test)]
#[path = "tests/selection_delta.rs"]
mod tests;
