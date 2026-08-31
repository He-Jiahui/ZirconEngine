use std::sync::Arc;

use crate::core::manager::{resolve_manager_service, ManagerServiceHandle};
#[cfg(test)]
use crate::core::TaskPool;
#[cfg(test)]
use crate::core::{
    manager::{manager_service_handle, RegisteredManagerService},
    runtime::ServiceObject,
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};
use crate::core::{CoreError, CoreHandle, CoreWeak};

use super::ProjectAssetManager;

#[derive(Clone, Debug)]
pub struct ProjectAssetManagerAccess {
    core: CoreWeak,
    handle: ManagerServiceHandle<ProjectAssetManager>,
    #[cfg(test)]
    _test_runtime: Option<CoreRuntime>,
}

impl ProjectAssetManagerAccess {
    pub fn new(core: CoreHandle, handle: ManagerServiceHandle<ProjectAssetManager>) -> Self {
        Self {
            core: core.downgrade(),
            handle,
            #[cfg(test)]
            _test_runtime: None,
        }
    }

    /// Builds an explicit, real CoreRuntime owner for graphics unit tests.
    ///
    /// Production callers must obtain the versioned handle from their owning
    /// runtime. Keeping this constructor test-only prevents a standalone
    /// manager path from becoming part of the engine architecture.
    #[cfg(test)]
    pub(crate) fn for_test(manager: Arc<ProjectAssetManager>) -> Self {
        const MODULE_NAME: &str = "TestProjectAssetRuntime";
        const SERVICE_NAME: &str = "TestProjectAssetRuntime.Manager.ProjectAssetManager";

        let runtime = CoreRuntime::new();
        runtime
            .register_module(
                ModuleDescriptor::new(MODULE_NAME, "test project asset runtime").with_manager(
                    ManagerDescriptor::new(
                        RegistryName::from_parts(
                            MODULE_NAME,
                            ServiceKind::Manager,
                            "ProjectAssetManager",
                        ),
                        StartupMode::Immediate,
                        Vec::new(),
                        Arc::new(move |_| {
                            Ok(
                                Arc::new(RegisteredManagerService::new(Arc::clone(&manager)))
                                    as ServiceObject,
                            )
                        }),
                    ),
                ),
            )
            .expect("test ProjectAssetManager service should register");
        runtime
            .activate_module(MODULE_NAME)
            .expect("test ProjectAssetManager module should activate");
        let core = runtime.handle();
        let handle = manager_service_handle(&core, SERVICE_NAME)
            .expect("test ProjectAssetManager handle should resolve");

        Self {
            core: core.downgrade(),
            handle,
            _test_runtime: Some(runtime),
        }
    }

    #[cfg(test)]
    pub(crate) fn test_worker_pool(&self) -> TaskPool {
        self._test_runtime
            .as_ref()
            .expect("test asset access must retain its CoreRuntime owner")
            .task_graph()
            .worker_pool()
            .clone()
    }

    pub fn resolve(&self) -> Result<Arc<ProjectAssetManager>, CoreError> {
        let core = self
            .core
            .upgrade()
            .ok_or_else(|| CoreError::ServiceUnavailable(self.handle.service.to_string()))?;
        resolve_manager_service(&core, self.handle.clone())
    }
}
