use crate::core::framework::scene::EntityPath;
use crate::scene::EntityId;

use super::super::World;

impl World {
    pub fn entity_path(&self, entity: EntityId) -> Option<EntityPath> {
        if !self.contains_entity(entity) {
            return None;
        }

        let mut segments = Vec::with_capacity(self.entity_path_segment_capacity(entity));
        let mut cursor = Some(entity);
        while let Some(current) = cursor {
            segments.push(self.path_segment_for_entity(current)?);
            cursor = self.parent_of(current);
        }
        segments.reverse();
        EntityPath::new(segments).ok()
    }

    pub fn resolve_entity_path(&self, path: &EntityPath) -> Option<EntityId> {
        let target_segments = path.segments();
        let mut entity_index = 0;
        while entity_index < self.entities.len() {
            let entity = self.entities[entity_index];
            if self.entity_matches_path_segments(entity, target_segments) {
                return Some(entity);
            }
            entity_index += 1;
        }

        None
    }

    fn entity_matches_path_segments(&self, entity: EntityId, target_segments: &[String]) -> bool {
        let mut cursor = Some(entity);
        let mut segment_index = target_segments.len();
        while let Some(current) = cursor {
            if segment_index == 0 {
                return false;
            }
            segment_index -= 1;
            let Some(segment) = self.path_segment_for_entity(current) else {
                return false;
            };
            if segment != target_segments[segment_index] {
                return false;
            }
            cursor = self.parent_of(current);
        }

        segment_index == 0
    }

    fn entity_path_segment_capacity(&self, entity: EntityId) -> usize {
        let mut capacity = 0;
        let mut cursor = Some(entity);
        while let Some(current) = cursor {
            capacity += 1;
            cursor = self.parent_of(current);
        }
        capacity
    }

    pub(super) fn path_segment_for_entity(&self, entity: EntityId) -> Option<String> {
        let name = self.names.get(&entity)?.0.trim();
        let base = if name.is_empty() {
            format!("Entity{entity}")
        } else {
            name.to_string()
        };
        Some(if self.entity_has_duplicate_path_name(entity, name) {
            format!("{base}#{entity}")
        } else {
            base
        })
    }

    fn entity_has_duplicate_path_name(&self, entity: EntityId, name: &str) -> bool {
        let parent = self.parent_of(entity);
        let mut candidate_index = 0;
        while candidate_index < self.entities.len() {
            let candidate = self.entities[candidate_index];
            candidate_index += 1;

            if candidate == entity || self.parent_of(candidate) != parent {
                continue;
            }
            let Some(candidate_name) = self.names.get(&candidate) else {
                continue;
            };
            if candidate_name.0.trim() == name {
                return true;
            }
        }

        false
    }
}
