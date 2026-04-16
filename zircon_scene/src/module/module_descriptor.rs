use std::sync::Arc;

use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_manager::LevelManagerHandle;
use zircon_module::{dependency_on, factory, qualified_name};

use super::manager_access::resolve_default_level_manager;
use super::{DefaultLevelManager, WorldDriver, SCENE_MODULE_NAME};

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
            let manager = resolve_default_level_manager(core)?;
            Ok(Arc::new(LevelManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}
