use std::sync::Arc;

use zircon_asset::{project::ProjectManager, AssetUri};
use zircon_core::{
    CoreError, CoreHandle, DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind,
    ServiceObject, StartupMode,
};
use zircon_manager::LevelManagerHandle;
use zircon_module::{dependency_on, factory, qualified_name, EngineModule};

mod core_error;
mod default_level_manager;
mod level_display_name;
mod level_manager_facade;
mod level_manager_lifecycle;
mod level_manager_project_io;
mod world_driver;

use core_error::scene_core_error;

pub use default_level_manager::DefaultLevelManager;
pub use world_driver::WorldDriver;

pub const SCENE_MODULE_NAME: &str = "SceneModule";
pub const WORLD_DRIVER_NAME: &str = "SceneModule.Driver.WorldDriver";
pub const DEFAULT_LEVEL_MANAGER_NAME: &str = "SceneModule.Manager.DefaultLevelManager";
pub const LEVEL_MANAGER_NAME: &str = zircon_manager::LEVEL_MANAGER_NAME;

#[derive(Clone, Copy, Debug, Default)]
pub struct SceneModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        SCENE_MODULE_NAME,
        "ECS world management, hierarchy, level lifecycle, and render extraction",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(SCENE_MODULE_NAME, ServiceKind::Driver, "WorldDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(WorldDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            SCENE_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultLevelManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(DefaultLevelManager::default()) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(SCENE_MODULE_NAME, ServiceKind::Manager, "LevelManager"),
        StartupMode::Immediate,
        vec![dependency_on(
            SCENE_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultLevelManager",
        )],
        factory(|core| {
            let manager =
                core.resolve_manager::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)?;
            Ok(Arc::new(LevelManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}

pub fn create_default_level(core: &CoreHandle) -> Result<crate::scene::LevelSystem, CoreError> {
    resolve_default_level_manager(core).map(|manager| manager.create_default_level())
}

pub fn load_level_asset(
    core: &CoreHandle,
    project_root: &str,
    uri: &str,
) -> Result<crate::scene::LevelSystem, CoreError> {
    let manager = resolve_default_level_manager(core)?;
    let mut project =
        ProjectManager::open(project_root).map_err(|error| scene_core_error(error.to_string()))?;
    project
        .scan_and_import()
        .map_err(|error| scene_core_error(error.to_string()))?;
    let uri = AssetUri::parse(uri).map_err(|error| scene_core_error(error.to_string()))?;
    manager
        .load_level(&project, &uri)
        .map_err(|error| scene_core_error(error.to_string()))
}

impl EngineModule for SceneModule {
    fn module_name(&self) -> &'static str {
        SCENE_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "ECS world management, hierarchy, level lifecycle, and render extraction"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

fn resolve_default_level_manager(core: &CoreHandle) -> Result<Arc<DefaultLevelManager>, CoreError> {
    core.resolve_manager::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)
}
