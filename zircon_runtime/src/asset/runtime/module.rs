use super::{runtime_asset_manager_handle, RuntimeAssetManager};
use crate::asset::{ASSET_MODULE_NAME, PROJECT_ASSET_MANAGER_NAME};
use crate::core::manager::{
    register_manager_alias, resolve_manager_service, RegisteredManagerService,
};
use crate::core::runtime::{modules::TASKS_MODULE_NAME, ServiceObject};
use crate::core::{
    CoreError, CoreResult, InitLevel, ManagerDescriptor, ModuleCleanupContext, ModuleContext,
    ModuleDependencySpec, ModuleDescriptor, ModuleLifecycle, ServiceKind, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name};
use std::sync::Arc;

#[derive(Debug)]
struct CookedAssetLifecycle;
impl ModuleLifecycle for CookedAssetLifecycle {
    fn ready(&self, context: &ModuleContext) -> CoreResult<bool> {
        let core = context
            .core
            .upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?;
        let manager = resolve_manager_service(&core, runtime_asset_manager_handle(&core)?)?;
        Ok(manager.catalog_generation_is_ready())
    }
    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> CoreResult<()> {
        context.with_existing_registered_manager::<RuntimeAssetManager>(
            PROJECT_ASSET_MANAGER_NAME,
            |manager| {
                manager.close();
                Ok(())
            },
        )?;
        Ok(())
    }
}
/// Registers the cooked reader and its existing ResourceManager alias in Core's service graph.
pub fn module_descriptor(manager: RuntimeAssetManager) -> ModuleDescriptor {
    ModuleDescriptor::new(ASSET_MODULE_NAME, "Read-only cooked asset residency")
        .with_init_level(InitLevel::Services)
        .with_module_dependency(ModuleDependencySpec::named(TASKS_MODULE_NAME))
        .with_lifecycle(Arc::new(CookedAssetLifecycle))
        .with_manager(ManagerDescriptor::new(
            qualified_name(
                ASSET_MODULE_NAME,
                ServiceKind::Manager,
                "ProjectAssetManager",
            ),
            StartupMode::Immediate,
            Vec::new(),
            factory(move |_| {
                Ok(
                    Arc::new(RegisteredManagerService::new(Arc::new(manager.clone())))
                        as ServiceObject,
                )
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
                let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
                register_manager_alias(
                    &core,
                    runtime_asset_manager_handle(&core)?,
                    crate::core::manager::RESOURCE_MANAGER_NAME,
                    |manager| manager as &dyn crate::core::framework::asset::ResourceManager,
                )
            }),
        ))
}
