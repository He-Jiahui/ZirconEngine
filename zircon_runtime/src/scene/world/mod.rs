//! ECS world state, project I/O, and render extraction.

mod bootstrap;
mod component_access;
mod derived_state;
mod dynamic_components;
mod hierarchy;
mod project_io;
mod property_access;
mod query;
mod records;
mod render;
mod world;

pub use project_io::SceneProjectError;
pub use world::World;
