//! Runtime scene subsystem: level orchestration plus the core ECS world.

mod level_system;
mod level_system_render_extract;
mod module;
mod runtime_level_traits;

pub use level_system::{LevelLifecycleState, LevelMetadata, LevelSystem};
pub use module::{
    create_default_level, load_level_asset, module_descriptor, DefaultLevelManager, SceneModule,
    WorldDriver, DEFAULT_LEVEL_MANAGER_NAME, LEVEL_MANAGER_NAME, SCENE_MODULE_NAME,
    WORLD_DRIVER_NAME,
};
pub use runtime_level_traits::{RuntimeObject, RuntimeSystem};

pub type EntityId = u64;
pub type NodeId = EntityId;

pub mod components;
mod render_extract;
pub mod semantics;
pub mod serializer;
pub mod world;

pub use world::World;

#[allow(unused_imports)]
pub(crate) use components::{
    default_render_layer_mask, Mobility, NodeKind, NodeRecord, Schedule, SystemStage,
};

pub type Scene = World;

#[cfg(test)]
mod tests;
