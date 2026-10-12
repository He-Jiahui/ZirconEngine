//! Runtime-owned scene reflection registry boundary.

#[path = "builtin_reflection/mod.rs"]
mod builtin_reflection;
#[path = "conversion.rs"]
mod conversion;
#[path = "declared_value_type.rs"]
mod declared_value_type;
#[path = "derived/mod.rs"]
mod derived;
#[path = "dynamic_component.rs"]
mod dynamic_component;
#[path = "dynamic_json.rs"]
mod dynamic_json;
#[path = "json_document/mod.rs"]
mod json_document;
#[path = "reflect_component.rs"]
mod reflect_component;
#[path = "reflect_resource.rs"]
mod reflect_resource;
#[path = "registration.rs"]
mod registration;
#[path = "runtime_type_registration.rs"]
mod runtime_type_registration;
#[path = "type_registry.rs"]
mod type_registry;
#[path = "value_admission.rs"]
mod value_admission;
#[path = "vm_type_backing.rs"]
mod vm_type_backing;
#[path = "world_reflection.rs"]
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
