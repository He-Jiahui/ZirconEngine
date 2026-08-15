use std::collections::HashMap;
use std::fmt;
#[cfg(test)]
use std::sync::Barrier;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::ThreadId;

use crate::core::diagnostics::RuntimeDevtoolsPluginCatalogEntry;
use crate::core::{CoreError, RuntimeModuleLifecycleObserver};

use super::super::descriptors::{FrozenModuleGraph, ModuleDescriptor, RegistryName};
use super::super::state::{
    CoreRuntimeInner, LifecycleCoordinator, ModuleEntry, ModuleLifecycleCommand,
    ModuleLifecycleTransitionPermit, ModuleLifecycleTransitionToken, ServiceEntry,
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

    pub(crate) fn lock_frozen_module_graph(
        &self,
    ) -> MutexGuard<'_, Option<Arc<FrozenModuleGraph>>> {
        lock_poison_recovered(&self.inner.frozen_module_graph)
    }

    fn lock_lifecycle_coordinator(&self) -> MutexGuard<'_, LifecycleCoordinator> {
        lock_poison_recovered(&self.inner.lifecycle_coordinator)
    }

    pub(crate) fn frozen_module_graph(&self) -> Result<Arc<FrozenModuleGraph>, CoreError> {
        let mut frozen_graph = self.lock_frozen_module_graph();
        if let Some(graph) = frozen_graph.as_ref() {
            return Ok(Arc::clone(graph));
        }

        let mut descriptors = self.registered_module_descriptors();
        descriptors.sort_by(|left, right| left.name.cmp(&right.name));
        let graph = Arc::new(FrozenModuleGraph::freeze(descriptors.as_slice())?);
        *frozen_graph = Some(Arc::clone(&graph));
        Ok(graph)
    }

    pub(crate) fn acquire_module_lifecycle_transition(
        &self,
        module_name: &str,
        command: ModuleLifecycleCommand,
    ) -> Result<ModuleLifecycleTransitionPermit, CoreError> {
        let owner = std::thread::current().id();
        let mut coordinator = self.lock_lifecycle_coordinator();
        loop {
            match coordinator.begin(module_name, command, owner)? {
                ModuleLifecycleTransitionPermit::Wait => {
                    coordinator = self
                        .inner
                        .lifecycle_transition_changed
                        .wait(coordinator)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                }
                permit => return Ok(permit),
            }
        }
    }

    pub(crate) fn complete_module_lifecycle_transition(
        &self,
        token: &ModuleLifecycleTransitionToken,
        result: Result<(), CoreError>,
    ) {
        let mut coordinator = self.lock_lifecycle_coordinator();
        coordinator.complete(token, result);
        drop(coordinator);
        self.inner.lifecycle_transition_changed.notify_all();
    }

    pub(crate) fn run_module_lifecycle_transition<F>(
        &self,
        module_name: &str,
        command: ModuleLifecycleCommand,
        operation: F,
    ) -> Result<(), CoreError>
    where
        F: FnOnce() -> Result<(), CoreError>,
    {
        match self.acquire_module_lifecycle_transition(module_name, command)? {
            ModuleLifecycleTransitionPermit::Completed(result) => result,
            ModuleLifecycleTransitionPermit::Owner(token) => {
                let result = operation();
                self.complete_module_lifecycle_transition(&token, result.clone());
                result
            }
            ModuleLifecycleTransitionPermit::Wait => {
                Err(CoreError::ModuleLifecycleCoordinatorUnresolved {
                    module: module_name.to_owned(),
                    command: command.as_str(),
                })
            }
        }
    }

    pub(crate) fn wait_for_service_resolution_change<'a>(
        &self,
        services: MutexGuard<'a, HashMap<RegistryName, ServiceEntry>>,
    ) -> MutexGuard<'a, HashMap<RegistryName, ServiceEntry>> {
        self.inner
            .service_resolution_changed
            .wait(services)
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn notify_service_resolution_changed(&self) {
        self.inner.service_resolution_changed.notify_all();
    }

    pub(crate) fn try_register_service_resolution_wait(
        &self,
        waiter: ThreadId,
        owner: ThreadId,
    ) -> bool {
        let mut waits = lock_poison_recovered(&self.inner.service_resolution_waits);
        waits.remove(&waiter);

        let mut cursor = owner;
        for _ in 0..=waits.len() {
            if cursor == waiter {
                return false;
            }
            let Some(next) = waits.get(&cursor).copied() else {
                waits.insert(waiter, owner);
                return true;
            };
            cursor = next;
        }

        false
    }

    pub(crate) fn clear_service_resolution_wait(&self, waiter: ThreadId) {
        lock_poison_recovered(&self.inner.service_resolution_waits).remove(&waiter);
    }

    pub(crate) fn register_service_activation_reentry(
        &self,
        owner: ThreadId,
        service: RegistryName,
    ) {
        lock_poison_recovered(&self.inner.service_activation_reentries).insert((owner, service));
    }

    pub(crate) fn take_service_activation_reentry(
        &self,
        owner: ThreadId,
        service: &RegistryName,
    ) -> bool {
        lock_poison_recovered(&self.inner.service_activation_reentries)
            .remove(&(owner, service.clone()))
    }

    pub(crate) fn clear_service_activation_reentry(&self, owner: ThreadId, service: &RegistryName) {
        lock_poison_recovered(&self.inner.service_activation_reentries)
            .remove(&(owner, service.clone()));
    }

    #[cfg(test)]
    pub(crate) fn install_service_resolution_claim_barrier(
        &self,
        claim_count: usize,
        barrier: Arc<Barrier>,
    ) {
        assert!(claim_count > 0);
        *lock_poison_recovered(&self.inner.service_resolution_claim_barrier) =
            Some((claim_count, barrier));
    }

    #[cfg(test)]
    pub(crate) fn wait_on_service_resolution_claim_barrier(&self) {
        let barrier = {
            let mut slot = lock_poison_recovered(&self.inner.service_resolution_claim_barrier);
            let Some((remaining, barrier)) = slot.as_mut() else {
                return;
            };
            let barrier = Arc::clone(barrier);
            *remaining -= 1;
            if *remaining == 0 {
                *slot = None;
            }
            barrier
        };
        barrier.wait();
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

    fn registered_module_descriptors(&self) -> Vec<ModuleDescriptor> {
        let modules = self.lock_modules();
        modules
            .values()
            .map(|entry| entry.descriptor().clone())
            .collect()
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
            let _guard = handle.inner.frozen_module_graph.lock().unwrap();
            panic!("poison core handle frozen module graph");
        }));
        assert!(handle.lock_frozen_module_graph().is_none());

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.lifecycle_coordinator.lock().unwrap();
            panic!("poison core handle lifecycle coordinator");
        }));
        let _ = handle.lock_lifecycle_coordinator();

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
