//! Scene module registration and level manager services.

mod core_error;
mod default_level_manager;
mod level_display_name;
mod level_manager_facade;
mod level_manager_lifecycle;
mod level_manager_project_io;
mod manager_access;
mod module_descriptor;
mod service_names;
mod world_driver;

pub use default_level_manager::DefaultLevelManager;
pub use manager_access::{create_default_level, load_level_asset};
pub use module_descriptor::module_descriptor;
pub use service_names::{
    DEFAULT_LEVEL_MANAGER_NAME, LEVEL_MANAGER_NAME, SCENE_MODULE_NAME, WORLD_DRIVER_NAME,
};
pub use world_driver::WorldDriver;
