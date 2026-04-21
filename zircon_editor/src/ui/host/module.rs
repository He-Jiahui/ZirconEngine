//! Editor module registration (Slint host shell).

use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::{ASSET_MODULE_NAME, PROJECT_ASSET_MANAGER_NAME};
use zircon_runtime::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name, EngineModule};
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;

use crate::ui::host::editor_asset_manager::{
    DefaultEditorAssetManager as EditorAssetManagerService, EditorAssetManagerHandle,
};
use crate::ui::host::EditorManager;

pub const EDITOR_MODULE_NAME: &str = "EditorModule";
pub const EDITOR_HOST_DRIVER_NAME: &str = "EditorModule.Driver.EditorHostDriver";
pub const EDITOR_MANAGER_NAME: &str = "EditorModule.Manager.EditorManager";
pub const EDITOR_ASSET_MANAGER_NAME: &str = "EditorModule.Manager.EditorAssetManager";

#[derive(Clone, Copy, Debug, Default)]
pub struct EditorModule;

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
        qualified_name(
            EDITOR_MODULE_NAME,
            ServiceKind::Manager,
            "EditorAssetManager",
        ),
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

impl EngineModule for EditorModule {
    fn module_name(&self) -> &'static str {
        EDITOR_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Slint-based editor host and tooling shell"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
