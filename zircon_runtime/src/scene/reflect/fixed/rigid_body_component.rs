use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::scene::{
    components::{RigidBodyComponent, RigidBodyType},
    reflect::ReflectComponent,
    reflect::TypeRegistry,
    EntityId, World,
};

use super::shared;

pub(super) const TYPE_PATH: &str = "zircon_runtime::scene::components::RigidBodyComponent";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(shared::component_registration(
        TYPE_PATH,
        "RigidBodyComponent",
        vec![
            shared::readonly_field("body_type", "Enum", ReflectEditorHint::Enum),
            shared::field("mass", "Scalar", ReflectEditorHint::Scalar),
            shared::field("linear_damping", "Scalar", ReflectEditorHint::Scalar),
            shared::field("angular_damping", "Scalar", ReflectEditorHint::Scalar),
            shared::field("gravity_scale", "Scalar", ReflectEditorHint::Scalar),
            shared::readonly_field("linear_velocity", "Vec3", ReflectEditorHint::Vec3),
            shared::readonly_field("angular_velocity", "Vec3", ReflectEditorHint::Vec3),
            shared::field("can_sleep", "Bool", ReflectEditorHint::Bool),
            shared::readonly_field("lock_translation", "List<Bool>", ReflectEditorHint::None),
            shared::readonly_field("lock_rotation", "List<Bool>", ReflectEditorHint::None),
        ],
        adapter(),
    ))
}

fn adapter() -> ReflectComponent {
    ReflectComponent::new(
        TYPE_PATH,
        contains,
        read_field,
        read_fields,
        write_field,
        remove,
    )
}

fn contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<RigidBodyComponent>(entity).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let component = shared::get_component::<RigidBodyComponent>(world, entity, TYPE_PATH)?;
    match field_name {
        "body_type" => Ok(ReflectedValue::Enum(body_type_name(component.body_type))),
        "mass" => Ok(ReflectedValue::Scalar(component.mass)),
        "linear_damping" => Ok(ReflectedValue::Scalar(component.linear_damping)),
        "angular_damping" => Ok(ReflectedValue::Scalar(component.angular_damping)),
        "gravity_scale" => Ok(ReflectedValue::Scalar(component.gravity_scale)),
        "linear_velocity" => Ok(ReflectedValue::Vec3(component.linear_velocity.to_array())),
        "angular_velocity" => Ok(ReflectedValue::Vec3(component.angular_velocity.to_array())),
        "can_sleep" => Ok(ReflectedValue::Bool(component.can_sleep)),
        "lock_translation" => Ok(bool_array_to_reflected_list(component.lock_translation)),
        "lock_rotation" => Ok(bool_array_to_reflected_list(component.lock_rotation)),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![
        ReflectFieldValue::new(
            "body_type",
            read_field(world, entity, TYPE_PATH, "body_type")?,
        ),
        ReflectFieldValue::new("mass", read_field(world, entity, TYPE_PATH, "mass")?),
        ReflectFieldValue::new(
            "linear_damping",
            read_field(world, entity, TYPE_PATH, "linear_damping")?,
        ),
        ReflectFieldValue::new(
            "angular_damping",
            read_field(world, entity, TYPE_PATH, "angular_damping")?,
        ),
        ReflectFieldValue::new(
            "gravity_scale",
            read_field(world, entity, TYPE_PATH, "gravity_scale")?,
        ),
        ReflectFieldValue::new(
            "linear_velocity",
            read_field(world, entity, TYPE_PATH, "linear_velocity")?,
        ),
        ReflectFieldValue::new(
            "angular_velocity",
            read_field(world, entity, TYPE_PATH, "angular_velocity")?,
        ),
        ReflectFieldValue::new(
            "can_sleep",
            read_field(world, entity, TYPE_PATH, "can_sleep")?,
        ),
        ReflectFieldValue::new(
            "lock_translation",
            read_field(world, entity, TYPE_PATH, "lock_translation")?,
        ),
        ReflectFieldValue::new(
            "lock_rotation",
            read_field(world, entity, TYPE_PATH, "lock_rotation")?,
        ),
    ])
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    shared::ensure_component::<RigidBodyComponent>(world, entity, TYPE_PATH)?;
    match field_name {
        "mass" => write_scalar(world, entity, field_name, value, |component, next| {
            component.mass = next;
        }),
        "linear_damping" => write_scalar(world, entity, field_name, value, |component, next| {
            component.linear_damping = next;
        }),
        "angular_damping" => write_scalar(world, entity, field_name, value, |component, next| {
            component.angular_damping = next;
        }),
        "gravity_scale" => write_scalar(world, entity, field_name, value, |component, next| {
            component.gravity_scale = next;
        }),
        "can_sleep" => {
            let next = shared::expect_bool(TYPE_PATH, field_name, value)?;
            if shared::get_component::<RigidBodyComponent>(world, entity, TYPE_PATH)?.can_sleep
                == next
            {
                return Ok(false);
            }
            let component =
                shared::get_component_mut::<RigidBodyComponent>(world, entity, TYPE_PATH)?;
            component.can_sleep = next;
            Ok(true)
        }
        "body_type" | "linear_velocity" | "angular_velocity" | "lock_translation"
        | "lock_rotation" => Err(shared::non_editable_field(TYPE_PATH, field_name)),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn write_scalar(
    world: &mut World,
    entity: EntityId,
    field_name: &str,
    value: ReflectedValue,
    apply: fn(&mut RigidBodyComponent, f32),
) -> Result<bool, ReflectError> {
    let next = shared::expect_scalar(TYPE_PATH, field_name, value)?;
    if read_scalar(world, entity, field_name)? == next {
        return Ok(false);
    }
    let component = shared::get_component_mut::<RigidBodyComponent>(world, entity, TYPE_PATH)?;
    apply(component, next);
    Ok(true)
}

fn read_scalar(world: &World, entity: EntityId, field_name: &str) -> Result<f32, ReflectError> {
    let component = shared::get_component::<RigidBodyComponent>(world, entity, TYPE_PATH)?;
    match field_name {
        "mass" => Ok(component.mass),
        "linear_damping" => Ok(component.linear_damping),
        "angular_damping" => Ok(component.angular_damping),
        "gravity_scale" => Ok(component.gravity_scale),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    shared::remove_component::<RigidBodyComponent>(world, entity, TYPE_PATH)
}

fn body_type_name(body_type: RigidBodyType) -> String {
    match body_type {
        RigidBodyType::Static => "Static".to_string(),
        RigidBodyType::Dynamic => "Dynamic".to_string(),
        RigidBodyType::Kinematic => "Kinematic".to_string(),
    }
}

fn bool_array_to_reflected_list(values: [bool; 3]) -> ReflectedValue {
    ReflectedValue::List(values.into_iter().map(ReflectedValue::Bool).collect())
}
