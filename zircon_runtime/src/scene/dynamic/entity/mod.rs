#[path = "dynamic_component.rs"]
mod dynamic_component;
#[path = "dynamic_entity.rs"]
mod dynamic_entity;
#[path = "dynamic_resource.rs"]
mod dynamic_resource;

pub use dynamic_component::DynamicComponent;
pub use dynamic_entity::DynamicEntity;
pub use dynamic_resource::DynamicResource;
