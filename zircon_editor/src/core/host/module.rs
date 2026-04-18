//! Editor module registration (Slint host shell).

use std::sync::Arc;

use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_foundation::FOUNDATION_MODULE_NAME;
use zircon_module::{dependency_on, factory, qualified_name};

use crate::manager::EditorManager;

pub const DEFAULT_PROJECT_PATH: &str = "sandbox-project";
pub(crate) const HISTORY_LIMIT: usize = 128;

pub const EDITOR_MODULE_NAME: &str = "EditorModule";
pub const EDITOR_HOST_DRIVER_NAME: &str = "EditorModule.Driver.EditorHostDriver";
pub const EDITOR_MANAGER_NAME: &str = "EditorModule.Manager.EditorManager";

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
}
