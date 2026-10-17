use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationStateMachinePlayerComponent,
};
use crate::scene::world::{SceneError, SceneResult, World};
use crate::scene::EntityId;

/// 编译绑定将这些运行时字段映射到动画组件；写入成功后由 World 统一标记节点缓存失效。
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
                update_scalar_field::<AnimationPlayerComponent>(
                    self,
                    entity,
                    "AnimationPlayer",
                    |player| player.playback_speed,
                    |player, next| player.playback_speed = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::PlayerTimeSeconds => {
                update_scalar_field::<AnimationPlayerComponent>(
                    self,
                    entity,
                    "AnimationPlayer",
                    |player| player.time_seconds,
                    |player, next| player.time_seconds = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::PlayerWeight => {
                update_scalar_field::<AnimationPlayerComponent>(
                    self,
                    entity,
                    "AnimationPlayer",
                    |player| player.weight,
                    |player, next| player.weight = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::PlayerLooping => {
                update_bool_field::<AnimationPlayerComponent>(
                    self,
                    entity,
                    "AnimationPlayer",
                    |player| player.looping,
                    |player, next| player.looping = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::PlayerPlaying => {
                update_bool_field::<AnimationPlayerComponent>(
                    self,
                    entity,
                    "AnimationPlayer",
                    |player| player.playing,
                    |player, next| player.playing = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::SequencePlayerPlaybackSpeed => {
                update_scalar_field::<AnimationSequencePlayerComponent>(
                    self,
                    entity,
                    "AnimationSequencePlayer",
                    |player| player.playback_speed,
                    |player, next| player.playback_speed = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::SequencePlayerTimeSeconds => {
                update_scalar_field::<AnimationSequencePlayerComponent>(
                    self,
                    entity,
                    "AnimationSequencePlayer",
                    |player| player.time_seconds,
                    |player, next| player.time_seconds = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::SequencePlayerLooping => {
                update_bool_field::<AnimationSequencePlayerComponent>(
                    self,
                    entity,
                    "AnimationSequencePlayer",
                    |player| player.looping,
                    |player, next| player.looping = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::SequencePlayerPlaying => {
                update_bool_field::<AnimationSequencePlayerComponent>(
                    self,
                    entity,
                    "AnimationSequencePlayer",
                    |player| player.playing,
                    |player, next| player.playing = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::GraphPlayerPlaying => {
                update_bool_field::<AnimationGraphPlayerComponent>(
                    self,
                    entity,
                    "AnimationGraphPlayer",
                    |player| player.playing,
                    |player, next| player.playing = next,
                    value,
                    property_path,
                )
            }
            CompiledAnimationRuntimeProperty::StateMachinePlayerPlaying => {
                update_bool_field::<AnimationStateMachinePlayerComponent>(
                    self,
                    entity,
                    "AnimationStateMachinePlayer",
                    |player| player.playing,
                    |player, next| player.playing = next,
                    value,
                    property_path,
                )
            }
        }?;
        if changed {
            self.mark_node_cache_dirty();
        }
        Ok(changed)
    }
}

fn update_scalar_field<T: crate::scene::ecs::Component>(
    world: &mut World,
    entity: EntityId,
    component: &'static str,
    read: impl FnOnce(&T) -> crate::core::math::Real,
    assign: impl FnOnce(&mut T, crate::core::math::Real),
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<bool> {
    let Some(current) = world.get::<T>(entity) else {
        return missing_animation_component(entity, component);
    };
    let next = World::compiled_property_expect_scalar(value, property_path)?;
    if read(current) == next {
        return Ok(false);
    }
    let Some(current) = world.get_mut::<T>(entity) else {
        return missing_animation_component(entity, component);
    };
    assign(current, next);
    Ok(true)
}

fn update_bool_field<T: crate::scene::ecs::Component>(
    world: &mut World,
    entity: EntityId,
    component: &'static str,
    read: impl FnOnce(&T) -> bool,
    assign: impl FnOnce(&mut T, bool),
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<bool> {
    let Some(current) = world.get::<T>(entity) else {
        return missing_animation_component(entity, component);
    };
    let next = World::compiled_property_expect_bool(value, property_path)?;
    if read(current) == next {
        return Ok(false);
    }
    let Some(current) = world.get_mut::<T>(entity) else {
        return missing_animation_component(entity, component);
    };
    assign(current, next);
    Ok(true)
}

fn missing_animation_component(entity: EntityId, component: &'static str) -> SceneResult<bool> {
    Err(SceneError::MissingRequiredComponent {
        operation: "write compiled animation runtime property",
        entity,
        component,
    })
}
