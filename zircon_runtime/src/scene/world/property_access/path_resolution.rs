use crate::core::framework::scene::EntityPath;
use crate::scene::EntityId;

use super::super::World;

impl World {
    pub fn entity_path(&self, entity: EntityId) -> Option<EntityPath> {
        if !self.contains_entity(entity) {
            return None;
        }

        let mut segments = Vec::new();
        let mut cursor = Some(entity);
        while let Some(current) = cursor {
            segments.push(self.path_segment_for_entity(current)?);
            cursor = self.parent_of(current);
        }
        segments.reverse();
        EntityPath::new(segments).ok()
    }

    pub fn resolve_entity_path(&self, path: &EntityPath) -> Option<EntityId> {
        self.entities.iter().copied().find(|entity| {
            self.entity_path(*entity)
                .as_ref()
                .is_some_and(|candidate| candidate == path)
        })
    }

    pub(super) fn path_segment_for_entity(&self, entity: EntityId) -> Option<String> {
        let name = self.names.get(&entity)?.0.trim();
        let base = if name.is_empty() {
            format!("Entity{entity}")
        } else {
            name.to_string()
        };
        let parent = self.parent_of(entity);
        let duplicate_count = self
            .entities
            .iter()
            .copied()
            .filter(|candidate| self.parent_of(*candidate) == parent)
            .filter(|candidate| {
                self.names
                    .get(candidate)
                    .is_some_and(|candidate_name| candidate_name.0.trim() == name)
            })
            .count();
        Some(if duplicate_count > 1 {
            format!("{base}#{entity}")
        } else {
            base
        })
    }
}
