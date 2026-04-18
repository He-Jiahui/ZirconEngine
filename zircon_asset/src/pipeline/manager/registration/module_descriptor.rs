use std::sync::Arc;

use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_module::{dependency_on, factory, qualified_name};

use super::super::{
    AssetIoDriver, AssetManagerHandle, ProjectAssetManager, ASSET_MODULE_NAME,
    PROJECT_ASSET_MANAGER_NAME,
};
use super::service_names::DEFAULT_EDITOR_ASSET_MANAGER_NAME;
use crate::{DefaultEditorAssetManager as EditorAssetManagerService, EditorAssetManagerHandle};
use zircon_manager::ResourceManagerHandle;

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
            "DefaultEditorAssetManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(EditorAssetManagerService::default()) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        ),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultEditorAssetManager",
        )],
        factory(|core| {
            let editor_asset_manager = core
                .resolve_manager::<EditorAssetManagerService>(DEFAULT_EDITOR_ASSET_MANAGER_NAME)?;
            Ok(Arc::new(ProjectAssetManager::with_editor_asset_manager(
                std::thread::available_parallelism().map_or(2, |n| n.get().max(2) - 1),
                editor_asset_manager,
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
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "EditorAssetManager",
        ),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultEditorAssetManager",
        )],
        factory(|core| {
            let manager = core
                .resolve_manager::<EditorAssetManagerService>(DEFAULT_EDITOR_ASSET_MANAGER_NAME)?;
            Ok(Arc::new(EditorAssetManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}
