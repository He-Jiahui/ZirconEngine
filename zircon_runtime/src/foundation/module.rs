use std::path::PathBuf;
use std::sync::Arc;

use crate::core::framework::foundation::{ConfigManager, FOUNDATION_MODULE_NAME};
use crate::core::manager::{RegisteredManagerService, CONFIG_MANAGER_NAME};
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, InitLevel, ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode,
};
use crate::engine_module::{factory, qualified_name, EngineModule};

use super::DefaultConfigManager;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        FOUNDATION_MODULE_NAME,
        "Built-in runtime foundation services",
    )
    .with_init_level(InitLevel::Kernel)
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            FOUNDATION_MODULE_NAME,
            ServiceKind::Manager,
            "ConfigManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            let manager = Arc::new(DefaultConfigManager::new(&core)?);
            Ok(
                Arc::new(RegisteredManagerService::<dyn ConfigManager>::new(manager))
                    as ServiceObject,
            )
        }),
    ))
}

/// Binds one host-owned persistence file before the normal Foundation descriptor is registered.
/// Only the canonical ConfigManager factory changes; module lifecycle and dependencies remain intact.
pub fn bind_config_file_path(
    descriptor: &mut ModuleDescriptor,
    path: PathBuf,
) -> Result<(), CoreError> {
    DefaultConfigManager::validate_file_path(&path)?;
    if descriptor.name != FOUNDATION_MODULE_NAME
        || descriptor
            .managers
            .iter()
            .filter(|manager| manager.name.as_str() == CONFIG_MANAGER_NAME)
            .count()
            != 1
    {
        return Err(CoreError::Initialization(
            "foundation config persistence".to_owned(),
            "descriptor must own exactly one canonical Foundation ConfigManager".to_owned(),
        ));
    }
    let manager = descriptor
        .managers
        .iter_mut()
        .find(|manager| manager.name.as_str() == CONFIG_MANAGER_NAME)
        .expect("canonical ConfigManager was checked above");
    manager.factory = factory(move |core| {
        let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
        let manager = Arc::new(DefaultConfigManager::new_with_file_path(
            &core,
            path.clone(),
        )?);
        Ok(Arc::new(RegisteredManagerService::<dyn ConfigManager>::new(manager)) as ServiceObject)
    });
    Ok(())
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoundationModule;

impl EngineModule for FoundationModule {
    fn module_name(&self) -> &'static str {
        FOUNDATION_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Built-in runtime foundation services"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
