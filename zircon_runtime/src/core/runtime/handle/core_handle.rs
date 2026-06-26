use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::plugin::RuntimePluginBridgeLifecycleState;

use super::super::descriptors::RegistryName;
use super::super::state::CoreRuntimeInner;
use super::super::state::{
    ModuleEntry, SceneRuntimeHookSet, ServiceEntry, WorldRuntimeExtensionSet,
};
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

    pub(crate) fn lock_scene_hooks(&self) -> MutexGuard<'_, SceneRuntimeHookSet> {
        lock_poison_recovered(&self.inner.scene_hooks)
    }

    pub(crate) fn lock_world_extensions(&self) -> MutexGuard<'_, WorldRuntimeExtensionSet> {
        lock_poison_recovered(&self.inner.world_extensions)
    }

    pub(crate) fn lock_plugin_bridge_lifecycle(
        &self,
    ) -> MutexGuard<'_, Option<RuntimePluginBridgeLifecycleState>> {
        lock_poison_recovered(&self.inner.plugin_bridge_lifecycle)
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
            let _guard = handle.inner.scene_hooks.lock().unwrap();
            panic!("poison core handle scene hooks");
        }));
        let _scene_hooks = handle.lock_scene_hooks();
        drop(_scene_hooks);

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.world_extensions.lock().unwrap();
            panic!("poison core handle world extensions");
        }));
        let _world_extensions = handle.lock_world_extensions();
        drop(_world_extensions);

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.plugin_bridge_lifecycle.lock().unwrap();
            panic!("poison core handle plugin bridge lifecycle");
        }));
        assert!(handle.lock_plugin_bridge_lifecycle().is_none());
    }
}
