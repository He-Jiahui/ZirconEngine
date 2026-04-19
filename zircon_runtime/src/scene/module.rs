#[path = "module/mod.rs"]
mod absorbed;

pub use absorbed::{
    create_default_level, load_level_asset, module_descriptor, DefaultLevelManager, SceneModule,
    WorldDriver, DEFAULT_LEVEL_MANAGER_NAME, LEVEL_MANAGER_NAME, SCENE_MODULE_NAME,
    WORLD_DRIVER_NAME,
};
