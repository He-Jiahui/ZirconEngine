//! ECS world state, project I/O, and render extraction.

mod bootstrap;
mod derived_state;
mod hierarchy;
mod project_io;
mod query;
mod records;
mod render;
mod world;

pub use project_io::SceneProjectError;
pub use world::World;
