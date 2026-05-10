//! ECS world state, project I/O, and render extraction.

mod bootstrap;
mod change_detection;
mod commands;
mod component_access;
mod component_type_registry;
mod derived_state;
mod dirty_state;
mod dynamic_components;
mod events;
mod hierarchy;
mod identity;
mod project_io;
mod property_access;
mod query;
mod records;
mod render;
mod typed_api;
mod world;

pub use component_type_registry::ComponentTypeRegistry;
pub use dynamic_components::DynamicComponentInstance;
pub use project_io::SceneProjectError;
pub use world::World;
