use std::sync::Arc;

use crate::asset::{project::ProjectManager, AssetUri, ASSET_MODULE_NAME};
use crate::core::framework::scene::LevelManager;
use crate::core::manager::RegisteredManagerService;
use crate::core::runtime::modules::TIME_MODULE_NAME;
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, CoreHandle, DriverDescriptor, InitLevel, ManagerDescriptor, ModuleDependencySpec,
    ModuleDescriptor, ServiceKind, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name, EngineModule};

mod core_error;
mod default_level_manager;
mod level_display_name;
mod level_manager_contract;
mod level_manager_lifecycle;
mod level_manager_project_io;
mod world_driver;

use core_error::scene_core_error;

pub use default_level_manager::DefaultLevelManager;
pub use world_driver::WorldDriver;

pub const SCENE_MODULE_NAME: &str = "SceneModule";
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
        factory(|core| Ok(Arc::new(DefaultLevelManager::with_core(core)) as ServiceObject)),
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
            Ok(
                Arc::new(RegisteredManagerService::<dyn LevelManager>::new(manager))
                    as ServiceObject,
            )
        }),
    ))
}

pub fn install_scene_runtime_hooks(
    core: &CoreHandle,
    registrations: impl IntoIterator<Item = crate::scene::SceneRuntimeHookRegistration>,
) -> Result<(), CoreError> {
    let driver = core.resolve_driver::<WorldDriver>(WORLD_DRIVER_NAME)?;
    driver.install_scene_runtime_hooks(core, registrations)
}

pub fn scene_runtime_hooks_for_stage(
    core: &CoreHandle,
    stage: crate::core::framework::scene::SystemStage,
) -> Result<Vec<crate::scene::SceneRuntimeHookRegistration>, CoreError> {
    let driver = core.resolve_driver::<WorldDriver>(WORLD_DRIVER_NAME)?;
    Ok(driver.scene_runtime_hooks_for_stage(stage))
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

pub(crate) fn resolve_default_level_manager(
    core: &CoreHandle,
) -> Result<Arc<DefaultLevelManager>, CoreError> {
    core.resolve_manager::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)
}
