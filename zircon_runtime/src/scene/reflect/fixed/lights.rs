use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::core::math::Vec3;
use crate::scene::{
    components::{DirectionalLight, PointLight, SpotLight},
    reflect::ReflectComponent,
    reflect::TypeRegistry,
    EntityId, World,
};

use super::shared;

const DIRECTIONAL_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::DirectionalLight";
const POINT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::PointLight";
const SPOT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::SpotLight";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(shared::component_registration(
        DIRECTIONAL_LIGHT_TYPE_PATH,
        "DirectionalLight",
        vec![
            shared::field("direction", "Vec3", ReflectEditorHint::Vec3),
            shared::field("color", "Vec3", ReflectEditorHint::Vec3),
            shared::field("intensity", "Scalar", ReflectEditorHint::Scalar),
        ],
        directional_adapter(),
    ))?;
    registry.register(shared::component_registration(
        POINT_LIGHT_TYPE_PATH,
        "PointLight",
        vec![
            shared::field("color", "Vec3", ReflectEditorHint::Vec3),
            shared::field("intensity", "Scalar", ReflectEditorHint::Scalar),
            shared::field("range", "Scalar", ReflectEditorHint::Scalar),
        ],
        point_adapter(),
    ))?;
    registry.register(shared::component_registration(
        SPOT_LIGHT_TYPE_PATH,
        "SpotLight",
        vec![
            shared::field("direction", "Vec3", ReflectEditorHint::Vec3),
            shared::field("color", "Vec3", ReflectEditorHint::Vec3),
            shared::field("intensity", "Scalar", ReflectEditorHint::Scalar),
            shared::field("range", "Scalar", ReflectEditorHint::Scalar),
            shared::field("inner_angle_radians", "Scalar", ReflectEditorHint::Scalar),
            shared::field("outer_angle_radians", "Scalar", ReflectEditorHint::Scalar),
        ],
        spot_adapter(),
    ))
}

fn directional_adapter() -> ReflectComponent {
    ReflectComponent::new(
        DIRECTIONAL_LIGHT_TYPE_PATH,
        directional_contains,
        directional_read_field,
        directional_read_fields,
        directional_write_field,
        directional_remove,
    )
}

fn point_adapter() -> ReflectComponent {
    ReflectComponent::new(
        POINT_LIGHT_TYPE_PATH,
        point_contains,
        point_read_field,
        point_read_fields,
        point_write_field,
        point_remove,
    )
}

fn spot_adapter() -> ReflectComponent {
    ReflectComponent::new(
        SPOT_LIGHT_TYPE_PATH,
        spot_contains,
        spot_read_field,
        spot_read_fields,
        spot_write_field,
        spot_remove,
    )
}

fn directional_contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<DirectionalLight>(entity).is_some()
}

fn point_contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<PointLight>(entity).is_some()
}

fn spot_contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<SpotLight>(entity).is_some()
}

fn directional_read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let light =
        shared::get_component::<DirectionalLight>(world, entity, DIRECTIONAL_LIGHT_TYPE_PATH)?;
    match field_name {
        "direction" => Ok(ReflectedValue::Vec3(light.direction.to_array())),
        "color" => Ok(ReflectedValue::Vec3(light.color.to_array())),
        "intensity" => Ok(ReflectedValue::Scalar(light.intensity)),
        _ => Err(shared::unknown_field(
            DIRECTIONAL_LIGHT_TYPE_PATH,
            field_name,
        )),
    }
}

fn directional_read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![
        ReflectFieldValue::new(
            "direction",
            directional_read_field(world, entity, DIRECTIONAL_LIGHT_TYPE_PATH, "direction")?,
        ),
        ReflectFieldValue::new(
            "color",
            directional_read_field(world, entity, DIRECTIONAL_LIGHT_TYPE_PATH, "color")?,
        ),
        ReflectFieldValue::new(
            "intensity",
            directional_read_field(world, entity, DIRECTIONAL_LIGHT_TYPE_PATH, "intensity")?,
        ),
    ])
}

fn point_read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let light = shared::get_component::<PointLight>(world, entity, POINT_LIGHT_TYPE_PATH)?;
    match field_name {
        "color" => Ok(ReflectedValue::Vec3(light.color.to_array())),
        "intensity" => Ok(ReflectedValue::Scalar(light.intensity)),
        "range" => Ok(ReflectedValue::Scalar(light.range)),
        _ => Err(shared::unknown_field(POINT_LIGHT_TYPE_PATH, field_name)),
    }
}

fn point_read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![
        ReflectFieldValue::new(
            "color",
            point_read_field(world, entity, POINT_LIGHT_TYPE_PATH, "color")?,
        ),
        ReflectFieldValue::new(
            "intensity",
            point_read_field(world, entity, POINT_LIGHT_TYPE_PATH, "intensity")?,
        ),
        ReflectFieldValue::new(
            "range",
            point_read_field(world, entity, POINT_LIGHT_TYPE_PATH, "range")?,
        ),
    ])
}

fn spot_read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let light = shared::get_component::<SpotLight>(world, entity, SPOT_LIGHT_TYPE_PATH)?;
    match field_name {
        "direction" => Ok(ReflectedValue::Vec3(light.direction.to_array())),
        "color" => Ok(ReflectedValue::Vec3(light.color.to_array())),
        "intensity" => Ok(ReflectedValue::Scalar(light.intensity)),
        "range" => Ok(ReflectedValue::Scalar(light.range)),
        "inner_angle_radians" => Ok(ReflectedValue::Scalar(light.inner_angle_radians)),
        "outer_angle_radians" => Ok(ReflectedValue::Scalar(light.outer_angle_radians)),
        _ => Err(shared::unknown_field(SPOT_LIGHT_TYPE_PATH, field_name)),
    }
}

fn spot_read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![
        ReflectFieldValue::new(
            "direction",
            spot_read_field(world, entity, SPOT_LIGHT_TYPE_PATH, "direction")?,
        ),
        ReflectFieldValue::new(
            "color",
            spot_read_field(world, entity, SPOT_LIGHT_TYPE_PATH, "color")?,
        ),
        ReflectFieldValue::new(
            "intensity",
            spot_read_field(world, entity, SPOT_LIGHT_TYPE_PATH, "intensity")?,
        ),
        ReflectFieldValue::new(
            "range",
            spot_read_field(world, entity, SPOT_LIGHT_TYPE_PATH, "range")?,
        ),
        ReflectFieldValue::new(
            "inner_angle_radians",
            spot_read_field(world, entity, SPOT_LIGHT_TYPE_PATH, "inner_angle_radians")?,
        ),
        ReflectFieldValue::new(
            "outer_angle_radians",
            spot_read_field(world, entity, SPOT_LIGHT_TYPE_PATH, "outer_angle_radians")?,
        ),
    ])
}

fn directional_write_field(
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

fn point_write_field(
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

fn spot_write_field(
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

fn directional_remove(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    shared::remove_component::<DirectionalLight>(world, entity, DIRECTIONAL_LIGHT_TYPE_PATH)
}

fn point_remove(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    shared::remove_component::<PointLight>(world, entity, POINT_LIGHT_TYPE_PATH)
}

fn spot_remove(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    shared::remove_component::<SpotLight>(world, entity, SPOT_LIGHT_TYPE_PATH)
}
