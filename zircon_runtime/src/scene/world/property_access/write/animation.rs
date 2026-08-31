use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent,
};
use crate::scene::{EntityId, SceneResult};

use super::super::super::World;
use super::super::value_conversion::{
    expect_animation_parameter, expect_bool, expect_resource_id, expect_string,
    missing_component_error, set_animation_player_like_property, unknown_property_error,
};

impl World {
    pub(super) fn set_animation_skeleton_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(skeleton) = self.get_mut::<AnimationSkeletonComponent>(entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "skeleton" => {
                let next = expect_resource_id(value, property_path)?;
                if skeleton.skeleton.id() == next {
                    return Ok(false);
                }
                skeleton.skeleton = crate::core::resource::ResourceHandle::new(next);
            }
            _ => return unknown_property_error(property_path),
        }
        self.mark_node_cache_dirty();
        Ok(true)
    }

    pub(super) fn set_animation_player_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(player) = self.get_mut::<AnimationPlayerComponent>(entity) else {
            return missing_component_error(entity, property_path);
        };
        let changed = set_animation_player_like_property(
            segments,
            value,
            property_path,
            &mut player.clip,
            &mut player.playback_speed,
            &mut player.time_seconds,
            Some(&mut player.weight),
            &mut player.looping,
            &mut player.playing,
        )?;
        if changed {
            self.mark_node_cache_dirty();
        }
        Ok(changed)
    }

    pub(super) fn set_animation_sequence_player_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(player) = self.get_mut::<AnimationSequencePlayerComponent>(entity) else {
            return missing_component_error(entity, property_path);
        };
        let changed = set_animation_player_like_property(
            segments,
            value,
            property_path,
            &mut player.sequence,
            &mut player.playback_speed,
            &mut player.time_seconds,
            None,
            &mut player.looping,
            &mut player.playing,
        )?;
        if changed {
            self.mark_node_cache_dirty();
        }
        Ok(changed)
    }

    pub(super) fn set_animation_graph_player_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(player) = self.get_mut::<AnimationGraphPlayerComponent>(entity) else {
            return missing_component_error(entity, property_path);
        };
        let changed = match segments {
            [field] if field == "graph" => {
                let next = expect_resource_id(value, property_path)?;
                if player.graph.id() == next {
                    false
                } else {
                    player.graph = crate::core::resource::ResourceHandle::new(next);
                    true
                }
            }
            [field] if field == "playing" => {
                let next = expect_bool(value, property_path)?;
                if player.playing == next {
                    false
                } else {
                    player.playing = next;
                    true
                }
            }
            [parameters, key] if parameters == "parameters" => {
                let next = expect_animation_parameter(value, property_path)?;
                if player.parameters.get(key) == Some(&next) {
                    false
                } else {
                    player.parameters.insert(key.clone(), next);
                    true
                }
            }
            _ => return unknown_property_error(property_path),
        };
        if changed {
            self.mark_node_cache_dirty();
        }
        Ok(changed)
    }

    pub(super) fn set_animation_state_machine_player_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(player) = self.get_mut::<AnimationStateMachinePlayerComponent>(entity) else {
            return missing_component_error(entity, property_path);
        };
        let changed = match segments {
            [field] if field == "statemachine" => {
                let next = expect_resource_id(value, property_path)?;
                if player.state_machine.id() == next {
                    false
                } else {
                    player.state_machine = crate::core::resource::ResourceHandle::new(next);
                    true
                }
            }
            [field] if field == "playing" => {
                let next = expect_bool(value, property_path)?;
                if player.playing == next {
                    false
                } else {
                    player.playing = next;
                    true
                }
            }
            [field] if field == "activestate" => {
                let next = expect_string(value, property_path)?;
                let next = if next.is_empty() { None } else { Some(next) };
                if player.active_state == next {
                    false
                } else {
                    player.active_state = next;
                    true
                }
            }
            [parameters, key] if parameters == "parameters" => {
                let next = expect_animation_parameter(value, property_path)?;
                if player.parameters.get(key) == Some(&next) {
                    false
                } else {
                    player.parameters.insert(key.clone(), next);
                    true
                }
            }
            _ => return unknown_property_error(property_path),
        };
        if changed {
            self.mark_node_cache_dirty();
        }
        Ok(changed)
    }
}
