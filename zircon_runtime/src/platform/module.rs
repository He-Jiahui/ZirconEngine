use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{PlatformDriver, PlatformManager};
use crate::core::framework::foundation::FOUNDATION_MODULE_NAME;
use crate::core::framework::platform::{PreferenceStorage, PLATFORM_MODULE_NAME};
use crate::core::manager::RegisteredManagerService;
use crate::core::runtime::modules::TASKS_MODULE_NAME;
use crate::core::runtime::ServiceObject;
use crate::core::{CoreError, CoreResult, ModuleContext, ModuleLifecycle};
use crate::engine_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, InitLevel,
    ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor, ServiceKind, StartupMode,
};

pub const PLATFORM_DRIVER_NAME: &str = "PlatformModule.Driver.PlatformDriver";
const PREFERENCE_PERSISTENCE_SHUTDOWN_TIMEOUT: Duration = Duration::from_millis(250);

#[derive(Debug, Default)]
struct PlatformModuleLifecycle;

impl ModuleLifecycle for PlatformModuleLifecycle {
    fn cleanup(&self, context: &ModuleContext) -> CoreResult<()> {
        let core = context
            .core
            .upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?;
        let driver = core.resolve_driver::<PlatformDriver>(PLATFORM_DRIVER_NAME)?;
        let deadline = Instant::now()
            .checked_add(PREFERENCE_PERSISTENCE_SHUTDOWN_TIMEOUT)
            .ok_or_else(|| CoreError::ModuleCleanupTimeout {
                module: context.module_name.clone(),
                operation: "preference_persistence".to_owned(),
                budget: PREFERENCE_PERSISTENCE_SHUTDOWN_TIMEOUT,
                incomplete_entries: usize::MAX,
                failed: 0,
                cancelled: 0,
            })?;
        match driver.shutdown_preference_persistence_until(deadline) {
            Ok(_) => Ok(()),
            Err(report) => Err(CoreError::ModuleCleanupTimeout {
                module: context.module_name.clone(),
                operation: "preference_persistence".to_owned(),
                budget: PREFERENCE_PERSISTENCE_SHUTDOWN_TIMEOUT,
                incomplete_entries: report.incomplete_entries,
                failed: report.failed,
                cancelled: report.cancelled,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlatformModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        PLATFORM_MODULE_NAME,
        "Platform, windowing, and OS integration",
    )
    .with_lifecycle(Arc::new(PlatformModuleLifecycle))
    .with_init_level(InitLevel::Services)
    .with_module_dependency(ModuleDependencySpec::named(FOUNDATION_MODULE_NAME))
    .with_module_dependency(ModuleDependencySpec::named(TASKS_MODULE_NAME))
    .with_driver(DriverDescriptor::new(
        qualified_name(PLATFORM_MODULE_NAME, ServiceKind::Driver, "PlatformDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            Ok(Arc::new(PlatformDriver::with_io_task_pool(
                core.task_graph().worker_pool().clone(),
            )) as _)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            PLATFORM_MODULE_NAME,
            ServiceKind::Manager,
            "PlatformManager",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            PLATFORM_MODULE_NAME,
            ServiceKind::Driver,
            "PlatformDriver",
        )],
        factory(|core| {
            let driver = core.resolve_driver::<PlatformDriver>(PLATFORM_DRIVER_NAME)?;
            let manager = Arc::new(PlatformManager::new(driver));
            Ok(
                Arc::new(RegisteredManagerService::<dyn PreferenceStorage>::new(
                    manager,
                )) as ServiceObject,
            )
        }),
    ))
}

impl EngineModule for PlatformModule {
    fn module_name(&self) -> &'static str {
        PLATFORM_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Platform, windowing, and OS integration"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
