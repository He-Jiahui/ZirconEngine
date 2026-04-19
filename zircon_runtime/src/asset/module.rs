use std::sync::Arc;

use crate::asset::pipeline::manager::{AssetIoDriver, AssetManagerHandle, ProjectAssetManager};
use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use crate::core::manager::ResourceManagerHandle;
use crate::engine_module::{dependency_on, factory, qualified_name, EngineModule};

pub const ASSET_MODULE_NAME: &str = "AssetModule";
pub const ASSET_IO_DRIVER_NAME: &str = "AssetModule.Driver.AssetIoDriver";
pub const PROJECT_ASSET_MANAGER_NAME: &str = "AssetModule.Manager.ProjectAssetManager";
pub const ASSET_MANAGER_NAME: &str = "AssetModule.Manager.AssetManager";
pub const RESOURCE_MANAGER_NAME: &str = crate::core::manager::RESOURCE_MANAGER_NAME;

#[derive(Clone, Copy, Debug, Default)]
pub struct AssetModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        ASSET_MODULE_NAME,
        "Asynchronous asset I/O and CPU-side decoding",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(ASSET_MODULE_NAME, ServiceKind::Driver, "AssetIoDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(AssetIoDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| {
            Ok(Arc::new(ProjectAssetManager::new(
                std::thread::available_parallelism().map_or(2, |n| n.get().max(2) - 1),
            )) as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(ASSET_MODULE_NAME, ServiceKind::Manager, "AssetManager"),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        )],
        factory(|core| {
            let manager =
                core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)?;
            Ok(Arc::new(AssetManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(ASSET_MODULE_NAME, ServiceKind::Manager, "ResourceManager"),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        )],
        factory(|core| {
            let manager =
                core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)?;
            Ok(Arc::new(ResourceManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}

impl EngineModule for AssetModule {
    fn module_name(&self) -> &'static str {
        ASSET_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Project asset pipeline, import workers, and resource indexing"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
