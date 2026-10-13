//! 位姿宿主导出通过当前场景世界读写实体；脚本传入的目标与步长必须符合世界坐标和有效时间语义，写入最终由 World 校验。
use crate::core::framework::script::{ScriptHostCallFrame, ScriptHostError, ScriptHostValue};
use crate::core::math::{
    transform_to_mat4, try_affine_inverse, NumericPolicy, Quat, Transform, Vec3,
};
use crate::script::runtime_context_for_frame;

use super::error::{GameplayHostError, GameplayHostResult};
use super::navigation::navigation_next_point;
use super::values::{expect_entity, expect_float, expect_vec3_json, to_json_string, vec3_to_array};

pub(super) fn position_json(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    let position = runtime.level.with_world(|world| {
        world
            .world_transform(entity)
            .map(|transform| vec3_to_array(transform.translation))
    });
    Ok(ScriptHostValue::String(to_json_string(&position)?))
}

pub(super) fn position_axis(
    context: &ScriptHostCallFrame<'_>,
    axis: usize,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = runtime_context_for_frame(context)?;
    let position = runtime.level.with_world(|world| {
        world
            .world_transform(entity)
            .map(|transform| vec3_to_array(transform.translation))
            .unwrap_or([0.0, 0.0, 0.0])
    });
    Ok(ScriptHostValue::Float(f64::from(position[axis])))
}

pub(super) fn set_position_json(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let position = expect_vec3_json(context, 1)?;
    set_entity_position(context, entity, position)
}

pub(super) fn set_position(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let position = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    set_entity_position(context, entity, position)
}

pub(super) fn translate_json(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let delta = expect_vec3_json(context, 1)?;
    translate_entity(context, entity, delta)
}

pub(super) fn translate(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let delta = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    translate_entity(context, entity, delta)
}

pub(super) fn face_direction(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let direction = Vec3::new(expect_float(context, 1)?, 0.0, expect_float(context, 2)?);
    face_entity_direction(context, entity, direction)
}

pub(super) fn set_scale(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let scale = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    set_entity_scale(context, entity, scale)
}

pub(super) fn follow_position(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let offset = Vec3::new(
        expect_float(context, 2)?,
        expect_float(context, 3)?,
        expect_float(context, 4)?,
    );
    follow_entity_position(context, entity, target_entity, offset)
}

pub(super) fn move_towards_entity(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let speed = expect_float(context, 2)?;
    let dt = expect_float(context, 3)?;
    move_entity_towards_target(context, entity, target_entity, speed, dt, false)
}

pub(super) fn camera_follow(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let offset = Vec3::new(
        expect_float(context, 2)?,
        expect_float(context, 3)?,
        expect_float(context, 4)?,
    );
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            let Some(target) = world.world_transform(target_entity) else {
                return Err(GameplayHostError::missing_entity(
                    "camera follow target",
                    target_entity,
                ));
            };
            let eye = target.translation + offset;
            let focus = target.translation + Vec3::Y;
            update_world_transform(world, entity, Transform::looking_at(eye, focus, Vec3::Y))
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

pub(super) fn set_entity_position(
    context: &ScriptHostCallFrame<'_>,
    entity: u64,
    position: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            update_world_position(world, entity, position)
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

pub(super) fn translate_entity(
    context: &ScriptHostCallFrame<'_>,
    entity: u64,
    delta: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            let position = world
                .world_matrix(entity)
                .ok_or_else(|| GameplayHostError::missing_entity("translate", entity))?;
            update_world_position(world, entity, position.w_axis.truncate() + delta)
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

pub(super) fn face_entity_direction(
    context: &ScriptHostCallFrame<'_>,
    entity: u64,
    direction: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let planar = Vec3::new(direction.x, 0.0, direction.z);
    if planar.length_squared() <= f32::EPSILON {
        return Ok(ScriptHostValue::Bool(false));
    }
    let yaw = planar.x.atan2(-planar.z);
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            let mut transform = world
                .world_transform(entity)
                .ok_or_else(|| GameplayHostError::missing_entity("face direction for", entity))?;
            transform.rotation = Quat::from_rotation_y(yaw);
            update_world_transform(world, entity, transform)
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

pub(super) fn set_entity_scale(
    context: &ScriptHostCallFrame<'_>,
    entity: u64,
    scale: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            let mut transform = world
                .world_transform(entity)
                .ok_or_else(|| GameplayHostError::missing_entity("set scale for", entity))?;
            transform.scale = scale;
            update_world_transform(world, entity, transform)
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

pub(super) fn follow_entity_position(
    context: &ScriptHostCallFrame<'_>,
    entity: u64,
    target_entity: u64,
    offset: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            let Some(target) = world.world_transform(target_entity) else {
                return Err(GameplayHostError::missing_entity(
                    "follow target",
                    target_entity,
                ));
            };
            update_world_position(world, entity, target.translation + offset)
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

pub(super) fn move_entity_towards_target(
    context: &ScriptHostCallFrame<'_>,
    entity: u64,
    target_entity: u64,
    speed: f32,
    dt: f32,
    prefer_navigation: bool,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            let start = world
                .world_matrix(entity)
                .map(|matrix| matrix.w_axis.truncate())
                .ok_or_else(|| GameplayHostError::missing_entity("move source", entity))?;
            let target = world
                .world_matrix(target_entity)
                .map(|matrix| matrix.w_axis.truncate())
                .ok_or_else(|| GameplayHostError::missing_entity("move target", target_entity))?;
            let target = if prefer_navigation {
                navigation_next_point(runtime, start, target).unwrap_or(target)
            } else {
                target
            };
            let delta = target - start;
            let distance = delta.length();
            if distance <= f32::EPSILON {
                return Ok(false);
            }
            // TODO: [CR-SCRIPT-AUDIT-0001] 确认趋近移动是否允许负速度或负时间步长；当前入口未限符号，会朝目标反向移动，导航入口却收敛负速度；补充参数契约与边界测试。
            let step = delta.normalize() * (speed * dt).min(distance);
            update_world_position(world, entity, start + step)
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}

/// Writes an intended world-space transform through World::update_transform's local-space API.
///
/// The scene stores TRS transforms, so the inverse-parent matrix is decomposed back to local TRS
/// after applying it. Translation remains exact for affine parent transforms, including rotated
/// and non-uniformly scaled parents; rotations/scales follow the scene's matrix decomposition.
fn update_world_transform(
    world: &mut crate::scene::World,
    entity: u64,
    transform: Transform,
) -> GameplayHostResult<bool> {
    update_world_matrix(world, entity, transform_to_mat4(transform))
}

fn update_world_position(
    world: &mut crate::scene::World,
    entity: u64,
    position: Vec3,
) -> GameplayHostResult<bool> {
    let mut world_matrix = world
        .world_matrix(entity)
        .ok_or_else(|| GameplayHostError::missing_entity("update position for", entity))?;
    world_matrix.w_axis = position.extend(1.0);
    update_world_matrix(world, entity, world_matrix)
}

fn update_world_matrix(
    world: &mut crate::scene::World,
    entity: u64,
    world_matrix: crate::core::math::Mat4,
) -> GameplayHostResult<bool> {
    if world.world_matrix(entity).is_none() {
        return Err(GameplayHostError::missing_entity(
            "update transform for",
            entity,
        ));
    }

    let local_transform = if let Some(parent) = world.parent_of(entity) {
        let parent_matrix =
            world
                .world_matrix(parent)
                .ok_or(crate::scene::SceneError::MissingParent {
                    child: entity,
                    parent,
                })?;
        let inverse = try_affine_inverse(parent_matrix, NumericPolicy::STRICT)
            .map_err(|_| GameplayHostError::NonInvertibleParentTransform { entity, parent })?;
        let local_matrix = inverse * world_matrix;
        let (scale, rotation, translation) = local_matrix.to_scale_rotation_translation();
        let local_transform = Transform {
            translation,
            rotation,
            scale,
        };
        if !matrix_matches_trs(local_matrix, transform_to_mat4(local_transform)) {
            return Err(GameplayHostError::NonRepresentableParentTransform { entity, parent });
        }
        local_transform
    } else {
        let (scale, rotation, translation) = world_matrix.to_scale_rotation_translation();
        Transform {
            translation,
            rotation,
            scale,
        }
    };

    Ok(world.update_transform(entity, local_transform)?)
}

fn matrix_matches_trs(
    matrix: crate::core::math::Mat4,
    reconstructed: crate::core::math::Mat4,
) -> bool {
    let expected = matrix.to_cols_array();
    let actual = reconstructed.to_cols_array();
    for index in 0..expected.len() {
        if !expected[index].is_finite() || !actual[index].is_finite() {
            return false;
        }
        let tolerance = 1.0e-4 * expected[index].abs().max(actual[index].abs()).max(1.0);
        if (expected[index] - actual[index]).abs() > tolerance {
            return false;
        }
    }
    true
}
