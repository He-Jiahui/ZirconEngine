use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::scene::ScenePropertyValue;
use crate::scene::EntityId;
use crate::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent,
};

use super::super::super::World;

impl World {
    pub(super) fn visit_animation_property_entries<F>(
        &self,
        entity: EntityId,
        visitor: &mut F,
    ) -> bool
    where
        F: FnMut(&str, &mut dyn FnMut() -> ScenePropertyValue, bool) -> bool,
    {
        macro_rules! push_entry {
            ($path:expr, $value:expr, $animatable:expr $(,)?) => {
                let mut build_value = || $value;
                if !visitor($path, &mut build_value, $animatable) {
                    return false;
                }
            };
        }

        if let Some(skeleton) = self.get::<AnimationSkeletonComponent>(entity) {
            push_entry!(
                "AnimationSkeleton.skeleton",
                ScenePropertyValue::Resource(skeleton.skeleton.id().to_string()),
                false,
            );
        }
        if let Some(player) = self.get::<AnimationPlayerComponent>(entity) {
            push_entry!(
                "AnimationPlayer.clip",
                ScenePropertyValue::Resource(player.clip.id().to_string()),
                false,
            );
            push_entry!(
                "AnimationPlayer.playback_speed",
                ScenePropertyValue::Scalar(player.playback_speed),
                true,
            );
            push_entry!(
                "AnimationPlayer.time_seconds",
                ScenePropertyValue::Scalar(player.time_seconds),
                true,
            );
            push_entry!(
                "AnimationPlayer.weight",
                ScenePropertyValue::Scalar(player.weight),
                true,
            );
            push_entry!(
                "AnimationPlayer.looping",
                ScenePropertyValue::Bool(player.looping),
                false,
            );
            push_entry!(
                "AnimationPlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
        }
        if let Some(player) = self.get::<AnimationSequencePlayerComponent>(entity) {
            push_entry!(
                "AnimationSequencePlayer.sequence",
                ScenePropertyValue::Resource(player.sequence.id().to_string()),
                false,
            );
            push_entry!(
                "AnimationSequencePlayer.playback_speed",
                ScenePropertyValue::Scalar(player.playback_speed),
                true,
            );
            push_entry!(
                "AnimationSequencePlayer.time_seconds",
                ScenePropertyValue::Scalar(player.time_seconds),
                true,
            );
            push_entry!(
                "AnimationSequencePlayer.looping",
                ScenePropertyValue::Bool(player.looping),
                false,
            );
            push_entry!(
                "AnimationSequencePlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
        }
        if let Some(player) = self.get::<AnimationGraphPlayerComponent>(entity) {
            push_entry!(
                "AnimationGraphPlayer.graph",
                ScenePropertyValue::Resource(player.graph.id().to_string()),
                false,
            );
            push_entry!(
                "AnimationGraphPlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
            for (key, value) in player.parameters.as_map() {
                push_entry!(
                    &format!("AnimationGraphPlayer.parameters.{key}"),
                    ScenePropertyValue::AnimationParameter(value.clone()),
                    animation_parameter_is_animatable(value),
                );
            }
        }
        if let Some(player) = self.get::<AnimationStateMachinePlayerComponent>(entity) {
            push_entry!(
                "AnimationStateMachinePlayer.state_machine",
                ScenePropertyValue::Resource(player.state_machine.id().to_string()),
                false,
            );
            push_entry!(
                "AnimationStateMachinePlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
            push_entry!(
                "AnimationStateMachinePlayer.active_state",
                ScenePropertyValue::String(match &player.active_state {
                    Some(active_state) => active_state.clone(),
                    None => String::new(),
                }),
                false,
            );
            for (key, value) in player.parameters.as_map() {
                push_entry!(
                    &format!("AnimationStateMachinePlayer.parameters.{key}"),
                    ScenePropertyValue::AnimationParameter(value.clone()),
                    animation_parameter_is_animatable(value),
                );
            }
        }

        true
    }

    pub(super) fn animation_property_entry_capacity_hint(&self, entity: EntityId) -> usize {
        let mut capacity = 0;
        if self.contains_component::<AnimationSkeletonComponent>(entity) {
            capacity += 1;
        }
        if self.contains_component::<AnimationPlayerComponent>(entity) {
            capacity += 6;
        }
        if self.contains_component::<AnimationSequencePlayerComponent>(entity) {
            capacity += 5;
        }
        if let Some(player) = self.get::<AnimationGraphPlayerComponent>(entity) {
            capacity += 2 + player.parameters.len();
        }
        if let Some(player) = self.get::<AnimationStateMachinePlayerComponent>(entity) {
            capacity += 3 + player.parameters.len();
        }
        capacity
    }
}

fn animation_parameter_is_animatable(value: &AnimationParameterValue) -> bool {
    matches!(
        value,
        AnimationParameterValue::Integer(_)
            | AnimationParameterValue::Scalar(_)
            | AnimationParameterValue::Vec2(_)
            | AnimationParameterValue::Vec3(_)
            | AnimationParameterValue::Vec4(_)
    )
}
