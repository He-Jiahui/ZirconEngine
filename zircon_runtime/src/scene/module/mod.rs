use std::sync::Arc;

use crate::asset::ASSET_MODULE_NAME;
#[cfg(feature = "native-host")]
use crate::asset::{AssetManager, AssetUri};
use crate::core::framework::scene::{LevelManager, SCENE_MODULE_NAME};
use crate::core::runtime::modules::TIME_MODULE_NAME;
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, CoreHandle, DriverDescriptor, InitLevel, ManagerDescriptor, ModuleDependencySpec,
    ModuleDescriptor, ServiceKind, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name, EngineModule};

#[path = "core_error.rs"]
mod core_error;
#[path = "default_level_manager.rs"]
mod default_level_manager;
#[path = "level_display_name.rs"]
mod level_display_name;
#[path = "level_manager_contract.rs"]
mod level_manager_contract;
#[path = "level_manager_lifecycle.rs"]
mod level_manager_lifecycle;
#[cfg(feature = "native-host")]
#[path = "level_manager_project_io.rs"]
mod level_manager_project_io;
#[cfg(feature = "native-host")]
#[path = "scene_artifact_io.rs"]
mod scene_artifact_io;
#[path = "world_driver.rs"]
mod world_driver;

use core_error::scene_core_error;

pub use default_level_manager::DefaultLevelManager;
pub use level_manager_lifecycle::{PreparedLevel, PreparedLevelPublishError};
pub use world_driver::WorldDriver;

pub const WORLD_DRIVER_NAME: &str = "SceneModule.Driver.WorldDriver";
pub const DEFAULT_LEVEL_MANAGER_NAME: &str = "SceneModule.Manager.DefaultLevelManager";

#[derive(Clone, Copy, Debug, Default)]
pub struct SceneModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        SCENE_MODULE_NAME,
        "ECS world management, hierarchy, level lifecycle, and render extraction",
    )
    .with_init_level(InitLevel::Scene)
    .with_module_dependency(ModuleDependencySpec::named(ASSET_MODULE_NAME))
    .with_module_dependency(ModuleDependencySpec::named(TIME_MODULE_NAME))
    .with_driver(DriverDescriptor::new(
        qualified_name(SCENE_MODULE_NAME, ServiceKind::Driver, "WorldDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(WorldDriver::default()) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            SCENE_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultLevelManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            Ok(Arc::new(DefaultLevelManager::with_core(&core)) as ServiceObject)
        }),
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
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            core.register_service_alias::<DefaultLevelManager, dyn LevelManager>(
                DEFAULT_LEVEL_MANAGER_NAME,
                ServiceKind::Manager,
                crate::core::manager::LEVEL_MANAGER_NAME,
                ServiceKind::Manager,
                |manager| manager,
            )
        }),
    ))
}

pub fn install_world_runtime_extension_plan(
    core: &CoreHandle,
    plan: crate::scene::WorldRuntimeExtensionPlan,
) -> Result<(), CoreError> {
    let driver = core.resolve_driver::<WorldDriver>(WORLD_DRIVER_NAME)?;
    driver.install_world_runtime_extension_plan(plan)
}

pub fn create_default_level(core: &CoreHandle) -> Result<crate::scene::LevelSystem, CoreError> {
    let manager = resolve_default_level_manager(core)?;
    manager.try_create_default_level()
}

pub fn create_level(
    core: &CoreHandle,
    world: crate::scene::World,
    metadata: crate::scene::LevelMetadata,
) -> Result<crate::scene::LevelSystem, CoreError> {
    let manager = resolve_default_level_manager(core)?;
    manager.try_create_level(world, metadata)
}

pub fn prepare_level(
    core: &CoreHandle,
    world: crate::scene::World,
    metadata: crate::scene::LevelMetadata,
) -> Result<PreparedLevel, CoreError> {
    let manager = core.resolve_manager_handle::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)?;
    let level = {
        let admitted = manager.enter()?;
        admitted.try_prepare_level(world, metadata)?
    };
    Ok(PreparedLevel::new(manager, level))
}

#[cfg(feature = "native-host")]
pub fn load_level_asset(
    core: &CoreHandle,
    asset_manager: &dyn AssetManager,
    uri: &str,
) -> Result<crate::scene::LevelSystem, CoreError> {
    let manager = resolve_default_level_manager(core)?;
    let uri = AssetUri::parse(uri).map_err(|error| scene_core_error(error.to_string()))?;
    let project = asset_manager
        .current_project_snapshot()
        .ok_or_else(|| scene_core_error("AssetManager has no active project"))?;
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

pub(crate) fn resolve_default_level_manager(
    core: &CoreHandle,
) -> Result<crate::core::runtime::ServiceCallGuard<DefaultLevelManager>, CoreError> {
    core.resolve_manager::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)
}
