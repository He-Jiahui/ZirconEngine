use crate::core::framework::scene::EntityPath;
use crate::scene::EntityId;
use crate::scene::components::Name;

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

    pub fn get_entity_by_path(&self, path: &EntityPath) -> Option<EntityId> {
        self.record_scene_property_path_lookup();
        let target_segments = path.segments();
        for entity in self.stable_entity_ids() {
            self.record_scene_property_path_entity_visit();
            if self.entity_matches_path_segments(entity, target_segments) {
                return Some(entity);
            }
        }

        None
    }

    fn entity_matches_path_segments(&self, entity: EntityId, target_segments: &[String]) -> bool {
        let mut cursor = Some(entity);
        let mut segment_index = target_segments.len();
        while let Some(current) = cursor {
            self.record_scene_property_path_ancestor_visit();
            if segment_index == 0 {
                return false;
            }
            segment_index -= 1;
            if !self.entity_path_segment_matches(current, &target_segments[segment_index]) {
                return false;
            }
            cursor = self.parent_of(current);
        }

        segment_index == 0
    }

    fn entity_path_segment_matches(&self, entity: EntityId, target: &str) -> bool {
        let Some(name) = self.get::<Name>(entity) else {
            return false;
        };
        let name = name.0.trim();
        let duplicate = self.entity_has_duplicate_path_name(entity, name);
        if name.is_empty() {
            let Some(suffix) = target.strip_prefix("Entity") else {
                return false;
            };
            if duplicate {
                let Some((base_id, duplicate_id)) = suffix.split_once('#') else {
                    return false;
                };
                return decimal_entity_id_matches(base_id, entity)
                    && decimal_entity_id_matches(duplicate_id, entity);
            }
            return decimal_entity_id_matches(suffix, entity);
        }
        if !duplicate {
            return target == name;
        }
        let Some(duplicate_id) = target
            .strip_prefix(name)
            .and_then(|suffix| suffix.strip_prefix('#'))
        else {
            return false;
        };
        decimal_entity_id_matches(duplicate_id, entity)
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
        let name = self.get::<Name>(entity)?.0.trim();
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
        for candidate in self.stable_entity_ids() {
            self.record_scene_property_path_sibling_visit();

            if candidate == entity || self.parent_of(candidate) != parent {
                continue;
            }
            let Some(candidate_name) = self.get::<Name>(candidate) else {
                continue;
            };
            if candidate_name.0.trim() == name {
                return true;
            }
        }

        false
    }
}

fn decimal_entity_id_matches(text: &str, expected: EntityId) -> bool {
    let bytes = text.as_bytes();
    if bytes.is_empty() || (bytes.len() > 1 && bytes[0] == b'0') {
        return false;
    }
    let mut value = 0_u64;
    for byte in bytes.iter().copied() {
        if !byte.is_ascii_digit() {
            return false;
        }
        let Some(next) = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(byte - b'0')))
        else {
            return false;
        };
        value = next;
    }
    value == expected
}
