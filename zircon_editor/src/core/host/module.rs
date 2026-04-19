//! Editor module registration (Slint host shell).

use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::{ASSET_MODULE_NAME, PROJECT_ASSET_MANAGER_NAME};
use zircon_runtime::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name};
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;

use crate::core::host::asset_editor::{
    DefaultEditorAssetManager as EditorAssetManagerService, EditorAssetManagerHandle,
};
use crate::core::host::manager::EditorManager;

pub const DEFAULT_PROJECT_PATH: &str = "sandbox-project";
pub(crate) const HISTORY_LIMIT: usize = 128;

pub const EDITOR_MODULE_NAME: &str = "EditorModule";
pub const EDITOR_HOST_DRIVER_NAME: &str = "EditorModule.Driver.EditorHostDriver";
pub const EDITOR_MANAGER_NAME: &str = "EditorModule.Manager.EditorManager";
pub const EDITOR_ASSET_MANAGER_NAME: &str = "EditorModule.Manager.EditorAssetManager";

#[derive(Debug, Default)]
pub struct EditorHostDriver;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        EDITOR_MODULE_NAME,
        "Slint-based editor host and tooling shell",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(EDITOR_MODULE_NAME, ServiceKind::Driver, "EditorHostDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(EditorHostDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(EDITOR_MODULE_NAME, ServiceKind::Manager, "EditorManager"),
        StartupMode::Lazy,
        vec![dependency_on(
            FOUNDATION_MODULE_NAME,
            ServiceKind::Manager,
            "ConfigManager",
        )],
        factory(|core| Ok(Arc::new(EditorManager::new(core.clone())) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(EDITOR_MODULE_NAME, ServiceKind::Manager, "EditorAssetManager"),
        StartupMode::Lazy,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        )],
        factory(|core| {
            let project_assets =
                core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)?;
            let manager = Arc::new(EditorAssetManagerService::with_project_asset_manager(
                project_assets,
            ));
            Ok(Arc::new(EditorAssetManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}
