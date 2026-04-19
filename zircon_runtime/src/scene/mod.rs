//! Runtime scene module registration absorbed into the runtime layer.

mod level_system;
#[path = "module/mod.rs"]
mod module;
mod render_extract;
#[path = "semantics.rs"]
mod runtime_semantics;

pub use level_system::{LevelLifecycleState, LevelMetadata, LevelSystem};
pub use module::{
    create_default_level, load_level_asset, module_descriptor, DefaultLevelManager, SceneModule,
    WorldDriver, DEFAULT_LEVEL_MANAGER_NAME, LEVEL_MANAGER_NAME, SCENE_MODULE_NAME,
    WORLD_DRIVER_NAME,
};
pub use runtime_semantics::{RuntimeObject, RuntimeSystem};
pub use zircon_scene::components;
pub use zircon_scene::semantics;
pub use zircon_scene::serializer;
pub use zircon_scene::world;
pub type Scene = zircon_scene::world::World;

#[cfg(test)]
mod tests;
