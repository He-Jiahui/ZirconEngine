use crate::core::framework::script::{ScriptHostCallContext, ScriptHostError, ScriptHostValue};
use crate::core::math::{Quat, Transform, Vec3};
use crate::script::current_script_runtime_call_context;

use super::navigation::navigation_next_point;
use super::values::{expect_entity, expect_float, expect_vec3_json, to_json_string, vec3_to_array};

pub(super) fn position_json(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = current_script_runtime_call_context()?;
    let position = runtime.level.with_world(|world| {
        world
            .world_transform(entity)
            .map(|transform| vec3_to_array(transform.translation))
    });
    Ok(ScriptHostValue::String(to_json_string(&position)?))
}

pub(super) fn position_axis(
    context: &ScriptHostCallContext,
    axis: usize,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let runtime = current_script_runtime_call_context()?;
    let position = runtime.level.with_world(|world| {
        world
            .world_transform(entity)
            .map(|transform| vec3_to_array(transform.translation))
            .unwrap_or([0.0, 0.0, 0.0])
    });
    Ok(ScriptHostValue::Float(f64::from(position[axis])))
}

pub(super) fn set_position_json(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let position = expect_vec3_json(context, 1)?;
    set_entity_position(entity, position)
}

pub(super) fn set_position(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let position = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    set_entity_position(entity, position)
}

pub(super) fn translate_json(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let delta = expect_vec3_json(context, 1)?;
    translate_entity(entity, delta)
}

pub(super) fn translate(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let delta = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    translate_entity(entity, delta)
}

pub(super) fn face_direction(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let direction = Vec3::new(expect_float(context, 1)?, 0.0, expect_float(context, 2)?);
    face_entity_direction(entity, direction)
}

pub(super) fn set_scale(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let scale = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    set_entity_scale(entity, scale)
}

pub(super) fn follow_position(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let offset = Vec3::new(
        expect_float(context, 2)?,
        expect_float(context, 3)?,
        expect_float(context, 4)?,
    );
    follow_entity_position(entity, target_entity, offset)
}

pub(super) fn move_towards_entity(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let speed = expect_float(context, 2)?;
    let dt = expect_float(context, 3)?;
    move_entity_towards_target(entity, target_entity, speed, dt, false)
}

pub(super) fn camera_follow(
    context: &ScriptHostCallContext,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let target_entity = expect_entity(context, 1)?;
    let offset = Vec3::new(
        expect_float(context, 2)?,
        expect_float(context, 3)?,
        expect_float(context, 4)?,
    );
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let Some(target) = world.world_transform(target_entity) else {
            return Err(format!(
                "camera follow target entity {target_entity} is missing"
            ));
        };
        let eye = target.translation + offset;
        let focus = target.translation + Vec3::Y;
        world.update_transform(entity, Transform::looking_at(eye, focus, Vec3::Y))
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

pub(super) fn set_entity_position(
    entity: u64,
    position: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.translation = position;
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

pub(super) fn translate_entity(
    entity: u64,
    delta: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.translation += delta;
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

pub(super) fn face_entity_direction(
    entity: u64,
    direction: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let planar = Vec3::new(direction.x, 0.0, direction.z);
    if planar.length_squared() <= f32::EPSILON {
        return Ok(ScriptHostValue::Bool(false));
    }
    let yaw = planar.x.atan2(-planar.z);
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.rotation = Quat::from_rotation_y(yaw);
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

pub(super) fn set_entity_scale(
    entity: u64,
    scale: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.scale = scale;
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

pub(super) fn follow_entity_position(
    entity: u64,
    target_entity: u64,
    offset: Vec3,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let result = runtime.level.with_world_mut(|world| {
        let Some(target) = world.world_transform(target_entity) else {
            return Err(format!("follow target entity {target_entity} is missing"));
        };
        let mut transform = world
            .world_transform(entity)
            .unwrap_or_else(Transform::default);
        transform.translation = target.translation + offset;
        world.update_transform(entity, transform)
    });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::new)
}

pub(super) fn move_entity_towards_target(
    entity: u64,
    target_entity: u64,
    speed: f32,
    dt: f32,
    prefer_navigation: bool,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = current_script_runtime_call_context()?;
    let (start, target) = runtime.level.with_world(|world| {
        (
            world
                .world_transform(entity)
                .map(|transform| transform.translation),
            world
                .world_transform(target_entity)
                .map(|transform| transform.translation),
        )
    });
    let Some(start) = start else {
        return Err(ScriptHostError::new(format!("entity {entity} is missing")));
    };
    let Some(target) = target else {
        return Err(ScriptHostError::new(format!(
            "target entity {target_entity} is missing"
        )));
    };
    let target = if prefer_navigation {
        navigation_next_point(&runtime, start, target).unwrap_or(target)
    } else {
        target
    };
    let delta = target - start;
    let distance = delta.length();
    if distance <= f32::EPSILON {
        return Ok(ScriptHostValue::Bool(false));
    }
    let step = delta.normalize() * (speed * dt).min(distance);
    translate_entity(entity, step)
}
