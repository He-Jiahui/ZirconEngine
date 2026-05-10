//! Runtime-owned scene reflection registry boundary.

mod conversion;
mod dynamic_component;
mod fixed;
mod reflect_component;
mod reflect_resource;
mod registration;
mod type_registry;
mod world_reflection;

pub use conversion::{
    json_from_reflected, reflected_from_json, reflected_from_scene_value,
    scene_value_from_reflected,
};
pub(in crate::scene) use dynamic_component::{
    reflect_component_for_dynamic_descriptor, registration_from_component_descriptor,
};
pub use reflect_component::ReflectComponent;
pub use reflect_resource::ReflectResource;
pub(in crate::scene) use registration::register_builtin_reflection;
pub use type_registry::{RuntimeTypeRegistration, TypeRegistry};
pub use world_reflection::WorldReflection;
