use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::EntityId;
use crate::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationStateMachinePlayerComponent,
};
use crate::scene::world::{SceneError, SceneResult, World};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompiledAnimationRuntimeProperty {
    PlayerPlaybackSpeed,
    PlayerTimeSeconds,
    PlayerWeight,
    PlayerLooping,
    PlayerPlaying,
    SequencePlayerPlaybackSpeed,
    SequencePlayerTimeSeconds,
    SequencePlayerLooping,
    SequencePlayerPlaying,
    GraphPlayerPlaying,
    StateMachinePlayerPlaying,
}

impl CompiledAnimationRuntimeProperty {
    pub(super) fn from_canonical_key(key: &str) -> Option<Self> {
        match key {
            "animationplayer.playbackspeed" => Some(Self::PlayerPlaybackSpeed),
            "animationplayer.timeseconds" => Some(Self::PlayerTimeSeconds),
            "animationplayer.weight" => Some(Self::PlayerWeight),
            "animationplayer.looping" => Some(Self::PlayerLooping),
            "animationplayer.playing" => Some(Self::PlayerPlaying),
            "animationsequenceplayer.playbackspeed" => Some(Self::SequencePlayerPlaybackSpeed),
            "animationsequenceplayer.timeseconds" => Some(Self::SequencePlayerTimeSeconds),
            "animationsequenceplayer.looping" => Some(Self::SequencePlayerLooping),
            "animationsequenceplayer.playing" => Some(Self::SequencePlayerPlaying),
            "animationgraphplayer.playing" => Some(Self::GraphPlayerPlaying),
            "animationstatemachineplayer.playing" => Some(Self::StateMachinePlayerPlaying),
            _ => None,
        }
    }
}

impl World {
    pub(super) fn read_compiled_animation_runtime_property(
        &self,
        entity: EntityId,
        property: CompiledAnimationRuntimeProperty,
    ) -> Option<ScenePropertyValue> {
        match property {
            CompiledAnimationRuntimeProperty::PlayerPlaybackSpeed => {
                Some(ScenePropertyValue::Scalar(
                    self.get::<AnimationPlayerComponent>(entity)?.playback_speed,
                ))
            }
            CompiledAnimationRuntimeProperty::PlayerTimeSeconds => {
                Some(ScenePropertyValue::Scalar(
                    self.get::<AnimationPlayerComponent>(entity)?.time_seconds,
                ))
            }
            CompiledAnimationRuntimeProperty::PlayerWeight => Some(ScenePropertyValue::Scalar(
                self.get::<AnimationPlayerComponent>(entity)?.weight,
            )),
            CompiledAnimationRuntimeProperty::PlayerLooping => Some(ScenePropertyValue::Bool(
                self.get::<AnimationPlayerComponent>(entity)?.looping,
            )),
            CompiledAnimationRuntimeProperty::PlayerPlaying => Some(ScenePropertyValue::Bool(
                self.get::<AnimationPlayerComponent>(entity)?.playing,
            )),
            CompiledAnimationRuntimeProperty::SequencePlayerPlaybackSpeed => {
                Some(ScenePropertyValue::Scalar(
                    self.get::<AnimationSequencePlayerComponent>(entity)?
                        .playback_speed,
                ))
            }
            CompiledAnimationRuntimeProperty::SequencePlayerTimeSeconds => {
                Some(ScenePropertyValue::Scalar(
                    self.get::<AnimationSequencePlayerComponent>(entity)?
                        .time_seconds,
                ))
            }
            CompiledAnimationRuntimeProperty::SequencePlayerLooping => {
                Some(ScenePropertyValue::Bool(
                    self.get::<AnimationSequencePlayerComponent>(entity)?
                        .looping,
                ))
            }
            CompiledAnimationRuntimeProperty::SequencePlayerPlaying => {
                Some(ScenePropertyValue::Bool(
                    self.get::<AnimationSequencePlayerComponent>(entity)?
                        .playing,
                ))
            }
            CompiledAnimationRuntimeProperty::GraphPlayerPlaying => Some(ScenePropertyValue::Bool(
                self.get::<AnimationGraphPlayerComponent>(entity)?.playing,
            )),
            CompiledAnimationRuntimeProperty::StateMachinePlayerPlaying => {
                Some(ScenePropertyValue::Bool(
                    self.get::<AnimationStateMachinePlayerComponent>(entity)?
                        .playing,
                ))
            }
        }
    }

    pub(super) fn write_compiled_animation_runtime_property(
        &mut self,
        entity: EntityId,
        property: CompiledAnimationRuntimeProperty,
        property_path: &ComponentPropertyPath,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        let changed = match property {
            CompiledAnimationRuntimeProperty::PlayerPlaybackSpeed => {
                let Some(player) = self.get_mut::<AnimationPlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationPlayer");
                };
                update_scalar_field(&mut player.playback_speed, value, property_path)
            }
            CompiledAnimationRuntimeProperty::PlayerTimeSeconds => {
                let Some(player) = self.get_mut::<AnimationPlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationPlayer");
                };
                update_scalar_field(&mut player.time_seconds, value, property_path)
            }
            CompiledAnimationRuntimeProperty::PlayerWeight => {
                let Some(player) = self.get_mut::<AnimationPlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationPlayer");
                };
                update_scalar_field(&mut player.weight, value, property_path)
            }
            CompiledAnimationRuntimeProperty::PlayerLooping => {
                let Some(player) = self.get_mut::<AnimationPlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationPlayer");
                };
                update_bool_field(&mut player.looping, value, property_path)
            }
            CompiledAnimationRuntimeProperty::PlayerPlaying => {
                let Some(player) = self.get_mut::<AnimationPlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationPlayer");
                };
                update_bool_field(&mut player.playing, value, property_path)
            }
            CompiledAnimationRuntimeProperty::SequencePlayerPlaybackSpeed => {
                let Some(player) = self.get_mut::<AnimationSequencePlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationSequencePlayer");
                };
                update_scalar_field(&mut player.playback_speed, value, property_path)
            }
            CompiledAnimationRuntimeProperty::SequencePlayerTimeSeconds => {
                let Some(player) = self.get_mut::<AnimationSequencePlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationSequencePlayer");
                };
                update_scalar_field(&mut player.time_seconds, value, property_path)
            }
            CompiledAnimationRuntimeProperty::SequencePlayerLooping => {
                let Some(player) = self.get_mut::<AnimationSequencePlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationSequencePlayer");
                };
                update_bool_field(&mut player.looping, value, property_path)
            }
            CompiledAnimationRuntimeProperty::SequencePlayerPlaying => {
                let Some(player) = self.get_mut::<AnimationSequencePlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationSequencePlayer");
                };
                update_bool_field(&mut player.playing, value, property_path)
            }
            CompiledAnimationRuntimeProperty::GraphPlayerPlaying => {
                let Some(player) = self.get_mut::<AnimationGraphPlayerComponent>(entity) else {
                    return missing_animation_component(entity, "AnimationGraphPlayer");
                };
                update_bool_field(&mut player.playing, value, property_path)
            }
            CompiledAnimationRuntimeProperty::StateMachinePlayerPlaying => {
                let Some(player) = self.get_mut::<AnimationStateMachinePlayerComponent>(entity)
                else {
                    return missing_animation_component(entity, "AnimationStateMachinePlayer");
                };
                update_bool_field(&mut player.playing, value, property_path)
            }
        }?;
        if changed {
            self.mark_node_cache_dirty();
        }
        Ok(changed)
    }
}

fn update_scalar_field(
    field: &mut crate::core::math::Real,
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<bool> {
    let next = World::compiled_property_expect_scalar(value, property_path)?;
    if *field == next {
        return Ok(false);
    }
    *field = next;
    Ok(true)
}

fn update_bool_field(
    field: &mut bool,
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<bool> {
    let next = World::compiled_property_expect_bool(value, property_path)?;
    if *field == next {
        return Ok(false);
    }
    *field = next;
    Ok(true)
}

fn missing_animation_component(entity: EntityId, component: &'static str) -> SceneResult<bool> {
    Err(SceneError::MissingRequiredComponent {
        operation: "write compiled animation runtime property",
        entity,
        component,
    })
}
