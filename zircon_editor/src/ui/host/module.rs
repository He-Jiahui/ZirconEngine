//! Editor module registration (Retained host shell).

use std::sync::Arc;

use zircon_runtime::asset::ASSET_MODULE_NAME;
use zircon_runtime::core::framework::render::GRAPHICS_MODULE_NAME;
use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::core::manager::RegisteredManagerService;
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreError, DriverDescriptor, InitLevel, ManagerDescriptor, ModuleDependencySpec,
    ModuleDescriptor, ServiceKind, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name, EngineModule};
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::ui::UI_MODULE_NAME;

use crate::core::commands::EditorCommandRegistryHandle;
use crate::ui::host::editor_asset_manager::{
    DefaultEditorAssetManager as EditorAssetManagerService, EditorAssetManager,
};
use crate::ui::host::{EditorKeymapService, EditorManager};

use super::runtime_services::EditorProjectAssetRuntimeAccess;

pub const EDITOR_MODULE_NAME: &str = "EditorModule";
pub const EDITOR_HOST_DRIVER_NAME: &str = "EditorModule.Driver.EditorHostDriver";
pub const EDITOR_MANAGER_NAME: &str = "EditorModule.Manager.EditorManager";
pub const EDITOR_ASSET_MANAGER_NAME: &str = "EditorModule.Manager.EditorAssetManager";
pub const EDITOR_COMMAND_REGISTRY_NAME: &str = "EditorModule.Manager.EditorCommandRegistry";
pub const EDITOR_KEYMAP_NAME: &str = "EditorModule.Manager.EditorKeymap";

#[derive(Clone, Copy, Debug, Default)]
pub struct EditorModule;

#[derive(Debug, Default)]
pub struct EditorHostDriver;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        EDITOR_MODULE_NAME,
        "Retained-based editor host and tooling shell",
    )
    .with_init_level(InitLevel::Editor)
    .with_module_dependency(ModuleDependencySpec::named(FOUNDATION_MODULE_NAME))
    .with_module_dependency(ModuleDependencySpec::named(ASSET_MODULE_NAME))
    .with_module_dependency(ModuleDependencySpec::named(SCENE_MODULE_NAME))
    .with_module_dependency(ModuleDependencySpec::named(GRAPHICS_MODULE_NAME))
    .with_module_dependency(ModuleDependencySpec::named(UI_MODULE_NAME))
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
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            Ok(Arc::new(EditorManager::new(&core)?) as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            EDITOR_MODULE_NAME,
            ServiceKind::Manager,
            "EditorAssetManager",
        ),
        StartupMode::Lazy,
        vec![
            dependency_on(
                ASSET_MODULE_NAME,
                ServiceKind::Manager,
                "ProjectAssetManager",
            ),
            dependency_on(EDITOR_MODULE_NAME, ServiceKind::Manager, "EditorManager"),
        ],
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            let project_assets = EditorProjectAssetRuntimeAccess::new(&core)?;
            let editor_manager = core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?;
            let manager = Arc::new(EditorAssetManagerService::with_runtime_project_manager(
                project_assets,
                editor_manager.context().jobs().clone(),
                editor_manager.context().logs_handle(),
            ));
            Ok(
                Arc::new(RegisteredManagerService::<dyn EditorAssetManager>::new(
                    manager,
                )) as ServiceObject,
            )
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            EDITOR_MODULE_NAME,
            ServiceKind::Manager,
            "EditorCommandRegistry",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            EDITOR_MODULE_NAME,
            ServiceKind::Manager,
            "EditorManager",
        )],
        factory(|core| {
            let manager = core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?;
            Ok(Arc::new(manager.context().commands().clone()) as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(EDITOR_MODULE_NAME, ServiceKind::Manager, "EditorKeymap"),
        StartupMode::Lazy,
        vec![dependency_on(
            EDITOR_MODULE_NAME,
            ServiceKind::Manager,
            "EditorManager",
        )],
        factory(|core| {
            let manager = core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?;
            Ok(manager.keymap_service() as ServiceObject)
        }),
    ))
}

impl EngineModule for EditorModule {
    fn module_name(&self) -> &'static str {
        EDITOR_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Retained-based editor host and tooling shell"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
