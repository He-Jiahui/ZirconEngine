use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::scene::{
    components::{AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight},
    reflect::ReflectComponent,
    reflect::TypeRegistry,
    EntityId, World,
};

mod write_fields;

use self::write_fields::{
    ambient_write_field, directional_write_field, point_write_field, rect_write_field,
    spot_write_field,
};
use super::shared;

const AMBIENT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::AmbientLight";
const DIRECTIONAL_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::DirectionalLight";
const POINT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::PointLight";
const RECT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::RectLight";
const SPOT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::SpotLight";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(shared::component_registration(
        AMBIENT_LIGHT_TYPE_PATH,
        "AmbientLight",
        vec![
            shared::field("color", "Vec3", ReflectEditorHint::Vec3),
            shared::field("intensity", "Scalar", ReflectEditorHint::Scalar),
            shared::field(
                "affects_lightmapped_meshes",
                "Bool",
                ReflectEditorHint::Bool,
            ),
        ],
        ambient_adapter(),
    ))?;
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
        RECT_LIGHT_TYPE_PATH,
        "RectLight",
        vec![
            shared::field("color", "Vec3", ReflectEditorHint::Vec3),
            shared::field("intensity", "Scalar", ReflectEditorHint::Scalar),
            shared::field("range", "Scalar", ReflectEditorHint::Scalar),
            shared::field("size", "Vec2", ReflectEditorHint::Vec2),
        ],
        rect_adapter(),
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

fn ambient_adapter() -> ReflectComponent {
    ReflectComponent::new(
        AMBIENT_LIGHT_TYPE_PATH,
        ambient_contains,
        ambient_read_field,
        ambient_read_fields,
        ambient_write_field,
        ambient_remove,
    )
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

fn rect_adapter() -> ReflectComponent {
    ReflectComponent::new(
        RECT_LIGHT_TYPE_PATH,
        rect_contains,
        rect_read_field,
        rect_read_fields,
        rect_write_field,
        rect_remove,
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

fn ambient_contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<AmbientLight>(entity).is_some()
}

fn directional_contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<DirectionalLight>(entity).is_some()
}

fn point_contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<PointLight>(entity).is_some()
}

fn rect_contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<RectLight>(entity).is_some()
}

fn spot_contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<SpotLight>(entity).is_some()
}

fn ambient_read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let light = shared::get_component::<AmbientLight>(world, entity, AMBIENT_LIGHT_TYPE_PATH)?;
    match field_name {
        "color" => Ok(ReflectedValue::Vec3(light.color.to_array())),
        "intensity" => Ok(ReflectedValue::Scalar(light.intensity)),
        "affects_lightmapped_meshes" => Ok(ReflectedValue::Bool(light.affects_lightmapped_meshes)),
        _ => Err(shared::unknown_field(AMBIENT_LIGHT_TYPE_PATH, field_name)),
    }
}

fn ambient_read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![
        ReflectFieldValue::new(
            "color",
            ambient_read_field(world, entity, AMBIENT_LIGHT_TYPE_PATH, "color")?,
        ),
        ReflectFieldValue::new(
            "intensity",
            ambient_read_field(world, entity, AMBIENT_LIGHT_TYPE_PATH, "intensity")?,
        ),
        ReflectFieldValue::new(
            "affects_lightmapped_meshes",
            ambient_read_field(
                world,
                entity,
                AMBIENT_LIGHT_TYPE_PATH,
                "affects_lightmapped_meshes",
            )?,
        ),
    ])
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

fn rect_read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let light = shared::get_component::<RectLight>(world, entity, RECT_LIGHT_TYPE_PATH)?;
    match field_name {
        "color" => Ok(ReflectedValue::Vec3(light.color.to_array())),
        "intensity" => Ok(ReflectedValue::Scalar(light.intensity)),
        "range" => Ok(ReflectedValue::Scalar(light.range)),
        "size" => Ok(ReflectedValue::Vec2(light.size.to_array())),
        _ => Err(shared::unknown_field(RECT_LIGHT_TYPE_PATH, field_name)),
    }
}

fn rect_read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![
        ReflectFieldValue::new(
            "color",
            rect_read_field(world, entity, RECT_LIGHT_TYPE_PATH, "color")?,
        ),
        ReflectFieldValue::new(
            "intensity",
            rect_read_field(world, entity, RECT_LIGHT_TYPE_PATH, "intensity")?,
        ),
        ReflectFieldValue::new(
            "range",
            rect_read_field(world, entity, RECT_LIGHT_TYPE_PATH, "range")?,
        ),
        ReflectFieldValue::new(
            "size",
            rect_read_field(world, entity, RECT_LIGHT_TYPE_PATH, "size")?,
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

fn ambient_remove(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    shared::remove_component::<AmbientLight>(world, entity, AMBIENT_LIGHT_TYPE_PATH)
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

fn rect_remove(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    shared::remove_component::<RectLight>(world, entity, RECT_LIGHT_TYPE_PATH)
}

fn spot_remove(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    shared::remove_component::<SpotLight>(world, entity, SPOT_LIGHT_TYPE_PATH)
}
