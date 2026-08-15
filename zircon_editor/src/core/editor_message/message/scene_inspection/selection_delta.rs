use std::collections::BTreeSet;

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
    pub(super) fn coalesce_from(&mut self, previous: &Self) {
        if previous.requires_resync()
            || self.requires_resync()
            || self.previous_revision != Some(previous.revision)
        {
            *self = Self::resync_at(self.revision);
            return;
        }

        let mut added = previous
            .added_entities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut removed = previous
            .removed_entities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
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
        self.removed_entities = removed.into_iter().collect();
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
mod tests {
    use super::SceneInspectionSelectionDelta;

    #[test]
    fn latest_deltas_compose_relative_to_the_oldest_retained_revision() {
        let previous = SceneInspectionSelectionDelta::between(4, 5, vec![7, 9], vec![3]);
        let mut current = SceneInspectionSelectionDelta::between(5, 6, vec![3, 11], vec![9]);

        current.coalesce_from(&previous);

        assert_eq!(current.previous_revision(), Some(4));
        assert_eq!(current.revision(), 6);
        assert_eq!(current.added_entities(), &[7, 11]);
        assert!(current.removed_entities().is_empty());
    }
}
