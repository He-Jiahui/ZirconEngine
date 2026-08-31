//! Runtime-owned scene reflection registry boundary.

mod builtin_reflection;
mod conversion;
mod declared_value_type;
mod derived;
mod dynamic_component;
mod dynamic_json;
mod json_document;
mod reflect_component;
mod reflect_resource;
mod registration;
mod runtime_type_registration;
mod type_registry;
mod value_admission;
mod vm_type_backing;
mod world_reflection;

pub use conversion::{reflected_from_scene_value, scene_value_from_reflected};
pub use derived::{
    derived_component_registration, derived_component_registration_with_adapter, ZrReflect,
    ZrReflectValue,
};
pub(in crate::scene) use dynamic_component::{
    reflect_component_for_dynamic_descriptor, registration_from_component_descriptor,
};
pub(in crate::scene) use dynamic_json::{
    ensure_json_value_type, json_value_from_reflected, reflected_value_from_json,
};
pub use json_document::{json_from_reflected, reflected_from_json, ReflectedJsonError};
pub use reflect_component::ReflectComponent;
pub use reflect_resource::ReflectResource;
pub(crate) use registration::builtin_type_registry;
pub(in crate::scene) use registration::register_builtin_reflection;
pub use runtime_type_registration::RuntimeTypeRegistration;
pub use type_registry::TypeRegistry;
pub(crate) use value_admission::{
    validate_reflected_field_values, validate_reflected_value, RUNTIME_REFLECT_VALUE_BUDGET,
};
pub use vm_type_backing::VmTypeBacking;
pub use world_reflection::WorldReflection;
