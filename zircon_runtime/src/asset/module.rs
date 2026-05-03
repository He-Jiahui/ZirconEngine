use std::sync::Arc;

use crate::asset::pipeline::manager::{AssetIoDriver, AssetManagerHandle, ProjectAssetManager};
use crate::asset::AssetImporterRegistry;
use crate::core::manager::ResourceManagerHandle;
use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name, EngineModule};

pub const ASSET_MODULE_NAME: &str = "AssetModule";
pub const ASSET_IO_DRIVER_NAME: &str = "AssetModule.Driver.AssetIoDriver";
pub const PROJECT_ASSET_MANAGER_NAME: &str = "AssetModule.Manager.ProjectAssetManager";
pub const ASSET_MANAGER_NAME: &str = "AssetModule.Manager.AssetManager";
pub const RESOURCE_MANAGER_NAME: &str = crate::core::manager::RESOURCE_MANAGER_NAME;

#[derive(Clone, Debug, Default)]
pub struct AssetModule {
    asset_importers: AssetImporterRegistry,
}

impl AssetModule {
    pub fn with_asset_importers(asset_importers: AssetImporterRegistry) -> Self {
        Self { asset_importers }
    }
}

pub fn module_descriptor() -> ModuleDescriptor {
    module_descriptor_with_asset_importers(AssetImporterRegistry::default())
}

fn module_descriptor_with_asset_importers(
    asset_importers: AssetImporterRegistry,
) -> ModuleDescriptor {
    let manager_asset_importers = asset_importers.clone();
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
        factory(move |_| {
            let manager = ProjectAssetManager::new(
                std::thread::available_parallelism().map_or(2, |n| n.get().max(2) - 1),
            );
            for importer in manager_asset_importers.importers() {
                manager.register_asset_importer_arc(importer)?;
            }
            Ok(Arc::new(manager) as ServiceObject)
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
        module_descriptor_with_asset_importers(self.asset_importers.clone())
    }
}
