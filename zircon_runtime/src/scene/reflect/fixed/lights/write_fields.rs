use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use crate::core::math::{Vec2, Vec3};
use crate::scene::{
    components::{AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight},
    EntityId, World,
};

use super::super::shared;
use super::{
    AMBIENT_LIGHT_TYPE_PATH, DIRECTIONAL_LIGHT_TYPE_PATH, POINT_LIGHT_TYPE_PATH,
    RECT_LIGHT_TYPE_PATH, SPOT_LIGHT_TYPE_PATH,
};

pub(super) fn ambient_write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    shared::ensure_component::<AmbientLight>(world, entity, AMBIENT_LIGHT_TYPE_PATH)?;
    match field_name {
        "color" => write_ambient_vec3(
            world,
            entity,
            field_name,
            value,
            |light| light.color,
            |light, next| {
                light.color = next;
            },
        ),
        "intensity" => write_ambient_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.intensity,
            |light, next| {
                light.intensity = next;
            },
        ),
        "affects_lightmapped_meshes" => write_ambient_bool(
            world,
            entity,
            field_name,
            value,
            |light| light.affects_lightmapped_meshes,
            |light, next| {
                light.affects_lightmapped_meshes = next;
            },
        ),
        _ => Err(shared::unknown_field(AMBIENT_LIGHT_TYPE_PATH, field_name)),
    }
}

pub(super) fn directional_write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    shared::ensure_component::<DirectionalLight>(world, entity, DIRECTIONAL_LIGHT_TYPE_PATH)?;
    match field_name {
        "direction" => write_directional_vec3(
            world,
            entity,
            field_name,
            value,
            |light| light.direction,
            |light, next| {
                light.direction = next;
            },
        ),
        "color" => write_directional_vec3(
            world,
            entity,
            field_name,
            value,
            |light| light.color,
            |light, next| {
                light.color = next;
            },
        ),
        "intensity" => write_directional_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.intensity,
            |light, next| {
                light.intensity = next;
            },
        ),
        _ => Err(shared::unknown_field(
            DIRECTIONAL_LIGHT_TYPE_PATH,
            field_name,
        )),
    }
}

pub(super) fn point_write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    shared::ensure_component::<PointLight>(world, entity, POINT_LIGHT_TYPE_PATH)?;
    match field_name {
        "color" => write_point_vec3(
            world,
            entity,
            field_name,
            value,
            |light| light.color,
            |light, next| {
                light.color = next;
            },
        ),
        "intensity" => write_point_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.intensity,
            |light, next| {
                light.intensity = next;
            },
        ),
        "range" => write_point_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.range,
            |light, next| {
                light.range = next;
            },
        ),
        _ => Err(shared::unknown_field(POINT_LIGHT_TYPE_PATH, field_name)),
    }
}

pub(super) fn rect_write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    shared::ensure_component::<RectLight>(world, entity, RECT_LIGHT_TYPE_PATH)?;
    match field_name {
        "color" => write_rect_vec3(
            world,
            entity,
            field_name,
            value,
            |light| light.color,
            |light, next| {
                light.color = next;
            },
        ),
        "intensity" => write_rect_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.intensity,
            |light, next| {
                light.intensity = next;
            },
        ),
        "range" => write_rect_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.range,
            |light, next| {
                light.range = next;
            },
        ),
        "size" => write_rect_vec2(
            world,
            entity,
            field_name,
            value,
            |light| light.size,
            |light, next| {
                light.size = next;
            },
        ),
        _ => Err(shared::unknown_field(RECT_LIGHT_TYPE_PATH, field_name)),
    }
}

pub(super) fn spot_write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    shared::ensure_component::<SpotLight>(world, entity, SPOT_LIGHT_TYPE_PATH)?;
    match field_name {
        "direction" => write_spot_vec3(
            world,
            entity,
            field_name,
            value,
            |light| light.direction,
            |light, next| {
                light.direction = next;
            },
        ),
        "color" => write_spot_vec3(
            world,
            entity,
            field_name,
            value,
            |light| light.color,
            |light, next| {
                light.color = next;
            },
        ),
        "intensity" => write_spot_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.intensity,
            |light, next| {
                light.intensity = next;
            },
        ),
        "range" => write_spot_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.range,
            |light, next| {
                light.range = next;
            },
        ),
        "inner_angle_radians" => write_spot_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.inner_angle_radians,
            |light, next| {
                light.inner_angle_radians = next;
            },
        ),
        "outer_angle_radians" => write_spot_scalar(
            world,
            entity,
            field_name,
            value,
            |light| light.outer_angle_radians,
            |light, next| {
                light.outer_angle_radians = next;
            },
        ),
        _ => Err(shared::unknown_field(SPOT_LIGHT_TYPE_PATH, field_name)),
    }
}

fn write_ambient_vec3(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&AmbientLight) -> Vec3,
    apply: fn(&mut AmbientLight, Vec3),
) -> Result<bool, ReflectError> {
    let next = Vec3::from_array(shared::expect_vec3(
        AMBIENT_LIGHT_TYPE_PATH,
        field_name,
        value,
    )?);
    if read(shared::get_component::<AmbientLight>(
        world,
        entity,
        AMBIENT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<AmbientLight>(world, entity, AMBIENT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_ambient_scalar(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&AmbientLight) -> f32,
    apply: fn(&mut AmbientLight, f32),
) -> Result<bool, ReflectError> {
    let next = shared::expect_scalar(AMBIENT_LIGHT_TYPE_PATH, field_name, value)?;
    if read(shared::get_component::<AmbientLight>(
        world,
        entity,
        AMBIENT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<AmbientLight>(world, entity, AMBIENT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_ambient_bool(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&AmbientLight) -> bool,
    apply: fn(&mut AmbientLight, bool),
) -> Result<bool, ReflectError> {
    let next = shared::expect_bool(AMBIENT_LIGHT_TYPE_PATH, field_name, value)?;
    if read(shared::get_component::<AmbientLight>(
        world,
        entity,
        AMBIENT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<AmbientLight>(world, entity, AMBIENT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_directional_vec3(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&DirectionalLight) -> Vec3,
    apply: fn(&mut DirectionalLight, Vec3),
) -> Result<bool, ReflectError> {
    let next = Vec3::from_array(shared::expect_vec3(
        DIRECTIONAL_LIGHT_TYPE_PATH,
        field_name,
        value,
    )?);
    if read(shared::get_component::<DirectionalLight>(
        world,
        entity,
        DIRECTIONAL_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light =
        shared::get_component_mut::<DirectionalLight>(world, entity, DIRECTIONAL_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_directional_scalar(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&DirectionalLight) -> f32,
    apply: fn(&mut DirectionalLight, f32),
) -> Result<bool, ReflectError> {
    let next = shared::expect_scalar(DIRECTIONAL_LIGHT_TYPE_PATH, field_name, value)?;
    if read(shared::get_component::<DirectionalLight>(
        world,
        entity,
        DIRECTIONAL_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light =
        shared::get_component_mut::<DirectionalLight>(world, entity, DIRECTIONAL_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_point_vec3(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&PointLight) -> Vec3,
    apply: fn(&mut PointLight, Vec3),
) -> Result<bool, ReflectError> {
    let next = Vec3::from_array(shared::expect_vec3(
        POINT_LIGHT_TYPE_PATH,
        field_name,
        value,
    )?);
    if read(shared::get_component::<PointLight>(
        world,
        entity,
        POINT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<PointLight>(world, entity, POINT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_point_scalar(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&PointLight) -> f32,
    apply: fn(&mut PointLight, f32),
) -> Result<bool, ReflectError> {
    let next = shared::expect_scalar(POINT_LIGHT_TYPE_PATH, field_name, value)?;
    if read(shared::get_component::<PointLight>(
        world,
        entity,
        POINT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<PointLight>(world, entity, POINT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_rect_vec3(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&RectLight) -> Vec3,
    apply: fn(&mut RectLight, Vec3),
) -> Result<bool, ReflectError> {
    let next = Vec3::from_array(shared::expect_vec3(
        RECT_LIGHT_TYPE_PATH,
        field_name,
        value,
    )?);
    if read(shared::get_component::<RectLight>(
        world,
        entity,
        RECT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<RectLight>(world, entity, RECT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_rect_vec2(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&RectLight) -> Vec2,
    apply: fn(&mut RectLight, Vec2),
) -> Result<bool, ReflectError> {
    let next = Vec2::from_array(shared::expect_vec2(
        RECT_LIGHT_TYPE_PATH,
        field_name,
        value,
    )?);
    if read(shared::get_component::<RectLight>(
        world,
        entity,
        RECT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<RectLight>(world, entity, RECT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_rect_scalar(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&RectLight) -> f32,
    apply: fn(&mut RectLight, f32),
) -> Result<bool, ReflectError> {
    let next = shared::expect_scalar(RECT_LIGHT_TYPE_PATH, field_name, value)?;
    if read(shared::get_component::<RectLight>(
        world,
        entity,
        RECT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<RectLight>(world, entity, RECT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_spot_vec3(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&SpotLight) -> Vec3,
    apply: fn(&mut SpotLight, Vec3),
) -> Result<bool, ReflectError> {
    let next = Vec3::from_array(shared::expect_vec3(
        SPOT_LIGHT_TYPE_PATH,
        field_name,
        value,
    )?);
    if read(shared::get_component::<SpotLight>(
        world,
        entity,
        SPOT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<SpotLight>(world, entity, SPOT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}

fn write_spot_scalar(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    read: fn(&SpotLight) -> f32,
    apply: fn(&mut SpotLight, f32),
) -> Result<bool, ReflectError> {
    let next = shared::expect_scalar(SPOT_LIGHT_TYPE_PATH, field_name, value)?;
    if read(shared::get_component::<SpotLight>(
        world,
        entity,
        SPOT_LIGHT_TYPE_PATH,
    )?) == next
    {
        return Ok(false);
    }
    let light = shared::get_component_mut::<SpotLight>(world, entity, SPOT_LIGHT_TYPE_PATH)?;
    apply(light, next);
    Ok(true)
}
