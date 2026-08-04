//! ECS world state, project I/O, and render extraction.

mod bootstrap;
mod change_detection;
mod commands;
mod compiled_binding;
mod component_access;
mod component_type_registry;
mod derived_state;
mod dirty_state;
mod dynamic_components;
mod error;
mod event_mirror;
mod events;
mod generation;
mod hierarchy;
mod identity;
mod messages;
mod observers;
mod performance_diagnostics;
mod project_io;
mod property_access;
mod query;
mod records;
mod render;
mod render_particles;
mod render_post_process;
mod render_visibility;
mod schedule;
mod staging_snapshot;
mod transaction;
mod transform_validation;
mod typed_api;
mod world;

pub use compiled_binding::{
    CompiledDescendantNameEntry, CompiledDescendantNameIndex, CompiledScenePropertyTarget,
    CompiledScenePropertyWriter, ComponentFieldId, PathId,
};
pub use component_type_registry::ComponentTypeRegistry;
pub use dynamic_components::DynamicComponentInstance;
pub(in crate::scene) use dynamic_components::json_from_scene_property_value;
pub use error::{SceneError, SceneResult};
pub use project_io::SceneProjectError;
pub use world::World;
