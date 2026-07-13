use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue, ZrReflectValue};

use crate::core::framework::scene::physics::{
    PhysicsCcdMode, PhysicsMassProperties, PhysicsSleepPolicy,
};

use super::super::{RigidBodyComponent, RigidBodyType};

const TYPE_PATH: &str = "zircon_runtime::scene::components::RigidBodyComponent";

pub(in crate::scene::components::scene) fn read_body_type(
    component: &RigidBodyComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Enum(
        match component.body_type {
            RigidBodyType::Static => "Static",
            RigidBodyType::Dynamic => "Dynamic",
            RigidBodyType::Kinematic => "Kinematic",
        }
        .to_string(),
    ))
}

pub(in crate::scene::components::scene) fn read_mass_properties_mode(
    component: &RigidBodyComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Enum(
        match component.mass_properties {
            PhysicsMassProperties::Explicit { .. } => "Explicit",
            PhysicsMassProperties::AutoFromShape { .. } => "AutoFromShape",
        }
        .to_string(),
    ))
}

pub(in crate::scene::components::scene) fn write_mass_properties_mode(
    component: &mut RigidBodyComponent,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let value = expect_enum(value, "mass_properties_mode")?;
    let next = match normalized_enum_name(&value).as_str() {
        "explicit" => PhysicsMassProperties::Explicit {
            inertia_tensor: None,
        },
        "autofromshape" => PhysicsMassProperties::AutoFromShape { density: 1.0 },
        _ => {
            return Err(type_mismatch(
                "mass_properties_mode",
                "Explicit or AutoFromShape Enum",
                "Enum",
            ))
        }
    };
    replace_if_changed(&mut component.mass_properties, next)
}

pub(in crate::scene::components::scene) fn read_mass_density(
    component: &RigidBodyComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Scalar(match component.mass_properties {
        PhysicsMassProperties::AutoFromShape { density } => density,
        PhysicsMassProperties::Explicit { .. } => 1.0,
    }))
}

pub(in crate::scene::components::scene) fn write_mass_density(
    component: &mut RigidBodyComponent,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let density = f32::from_reflected_value(value, TYPE_PATH, "mass_density")?;
    if density <= 0.0 {
        return Err(type_mismatch("mass_density", "positive Scalar", "Scalar"));
    }
    replace_if_changed(
        &mut component.mass_properties,
        PhysicsMassProperties::AutoFromShape { density },
    )
}

pub(in crate::scene::components::scene) fn read_linear_velocity(
    component: &RigidBodyComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(component.linear_velocity.to_reflected_value())
}

pub(in crate::scene::components::scene) fn read_angular_velocity(
    component: &RigidBodyComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(component.angular_velocity.to_reflected_value())
}

pub(in crate::scene::components::scene) fn read_ccd_mode(
    component: &RigidBodyComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Enum(
        match component.ccd_mode {
            PhysicsCcdMode::Disabled => "Disabled",
            PhysicsCcdMode::LinearCast => "LinearCast",
        }
        .to_string(),
    ))
}

pub(in crate::scene::components::scene) fn write_ccd_mode(
    component: &mut RigidBodyComponent,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let value = expect_enum(value, "ccd_mode")?;
    let next = match normalized_enum_name(&value).as_str() {
        "disabled" => PhysicsCcdMode::Disabled,
        "linearcast" => PhysicsCcdMode::LinearCast,
        _ => {
            return Err(type_mismatch(
                "ccd_mode",
                "Disabled or LinearCast Enum",
                "Enum",
            ))
        }
    };
    replace_if_changed(&mut component.ccd_mode, next)
}

pub(in crate::scene::components::scene) fn read_sleep_policy(
    component: &RigidBodyComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Enum(
        match component.sleep_policy {
            PhysicsSleepPolicy::Allow => "Allow",
            PhysicsSleepPolicy::Never => "Never",
        }
        .to_string(),
    ))
}

pub(in crate::scene::components::scene) fn write_sleep_policy(
    component: &mut RigidBodyComponent,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let value = expect_enum(value, "sleep_policy")?;
    let next = match normalized_enum_name(&value).as_str() {
        "allow" => PhysicsSleepPolicy::Allow,
        "never" => PhysicsSleepPolicy::Never,
        _ => return Err(type_mismatch("sleep_policy", "Allow or Never Enum", "Enum")),
    };
    replace_if_changed(&mut component.sleep_policy, next)
}

pub(in crate::scene::components::scene) fn read_lock_translation(
    component: &RigidBodyComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(bool_array(component.lock_translation))
}

pub(in crate::scene::components::scene) fn read_lock_rotation(
    component: &RigidBodyComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(bool_array(component.lock_rotation))
}

fn bool_array(values: [bool; 3]) -> ReflectedValue {
    let mut reflected = Vec::with_capacity(values.len());
    for value in values {
        reflected.push(ReflectedValue::Bool(value));
    }
    ReflectedValue::List(reflected)
}

fn expect_enum(value: ReflectedValue, field_name: &str) -> Result<String, ReflectError> {
    match value {
        ReflectedValue::Enum(value) => Ok(value),
        value => Err(type_mismatch(field_name, "Enum", value.type_name())),
    }
}

fn normalized_enum_name(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        }
    }
    normalized
}

fn replace_if_changed<T>(current: &mut T, next: T) -> Result<bool, ReflectError>
where
    T: PartialEq,
{
    if *current == next {
        return Ok(false);
    }
    *current = next;
    Ok(true)
}

fn type_mismatch(field_name: &str, expected: &str, actual: &str) -> ReflectError {
    ReflectError::TypeMismatch {
        type_path: TYPE_PATH.to_string(),
        field_name: field_name.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    }
}
