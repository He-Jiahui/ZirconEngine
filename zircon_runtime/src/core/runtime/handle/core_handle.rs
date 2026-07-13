use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::diagnostics::{
    RuntimeDevtoolsPluginCatalogEntry, RuntimeDevtoolsSceneHookSnapshot,
};
use crate::core::RuntimeModuleLifecycleObserver;

use super::super::descriptors::RegistryName;
use super::super::state::CoreRuntimeInner;
use super::super::state::{ModuleEntry, ServiceEntry};
use super::super::tasks::{JobScheduler, TaskPool, TaskPoolKind, TaskPoolReport, TaskPools};
use super::super::weak::CoreWeak;

#[derive(Clone)]
pub struct CoreHandle {
    pub(crate) inner: Arc<CoreRuntimeInner>,
}

impl CoreHandle {
    pub fn downgrade(&self) -> CoreWeak {
        CoreWeak {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn scheduler(&self) -> &JobScheduler {
        &self.inner.scheduler
    }

    pub fn task_pools(&self) -> &TaskPools {
        &self.inner.task_pools
    }

    pub fn task_pool(&self, kind: TaskPoolKind) -> &TaskPool {
        self.inner.task_pools.get(kind)
    }

    pub fn task_pool_report(&self) -> TaskPoolReport {
        self.inner.task_pools.report()
    }

    pub(crate) fn lock_modules(&self) -> MutexGuard<'_, HashMap<String, ModuleEntry>> {
        lock_poison_recovered(&self.inner.modules)
    }

    pub(crate) fn lock_services(&self) -> MutexGuard<'_, HashMap<RegistryName, ServiceEntry>> {
        lock_poison_recovered(&self.inner.services)
    }

    pub(crate) fn replace_devtools_scene_hook_snapshots(
        &self,
        snapshots: Vec<RuntimeDevtoolsSceneHookSnapshot>,
    ) {
        *lock_poison_recovered(&self.inner.scene_hook_snapshots) = snapshots;
    }

    pub fn replace_devtools_plugin_catalog_entries(
        &self,
        entries: Vec<RuntimeDevtoolsPluginCatalogEntry>,
    ) {
        *lock_poison_recovered(&self.inner.devtools_plugin_catalog_entries) = entries;
    }

    pub(crate) fn lock_runtime_module_lifecycle_observer(
        &self,
    ) -> MutexGuard<'_, Option<Arc<dyn RuntimeModuleLifecycleObserver>>> {
        lock_poison_recovered(&self.inner.runtime_module_lifecycle_observer)
    }
}

fn lock_poison_recovered<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl fmt::Debug for CoreHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreHandle").finish()
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};

    use crate::core::CoreRuntime;

    #[test]
    fn core_handle_registry_accessors_recover_poisoned_runtime_locks() {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.modules.lock().unwrap();
            panic!("poison core handle modules registry");
        }));
        assert!(handle.lock_modules().is_empty());

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.services.lock().unwrap();
            panic!("poison core handle services registry");
        }));
        assert!(handle.lock_services().is_empty());

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.scene_hook_snapshots.lock().unwrap();
            panic!("poison core handle scene hook diagnostics snapshots");
        }));
        handle.replace_devtools_scene_hook_snapshots(Vec::new());

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.devtools_plugin_catalog_entries.lock().unwrap();
            panic!("poison core handle devtools plugin catalog entries");
        }));
        handle.replace_devtools_plugin_catalog_entries(Vec::new());

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle
                .inner
                .runtime_module_lifecycle_observer
                .lock()
                .unwrap();
            panic!("poison core handle runtime module lifecycle observer");
        }));
        assert!(handle.lock_runtime_module_lifecycle_observer().is_none());
    }
}
