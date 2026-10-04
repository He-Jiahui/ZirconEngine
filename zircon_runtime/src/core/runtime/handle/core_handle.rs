use std::collections::HashMap;
use std::fmt;
#[cfg(test)]
use std::sync::Barrier;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

use crate::core::diagnostics::RuntimeDevtoolsPluginCatalogEntry;
use crate::core::{CoreError, RuntimeModuleLifecycleObserver};

use super::super::descriptors::{FrozenModuleGraph, ModuleDescriptor, RegistryName};
use super::super::state::{
    CoreRuntimeInner, LifecycleCoordinator, ModuleEntry, ModuleLifecycleCommand,
    ModuleLifecycleTransitionAdmission, ModuleLifecycleTransitionPermit,
    ModuleLifecycleTransitionToken, ServiceEntry,
};
use super::super::tasks::{EngineTaskGraph, JobScheduler, TaskGraphWorkerInventory};
use super::super::weak::CoreWeak;

/// 可克隆的运行时控制句柄；所有副本共享模块表、服务槽位和调度器。
/// 服务工厂或插件若只需回访内核，应传递 [`CoreWeak`] 以免延长其生命周期。
#[derive(Clone)]
pub struct CoreHandle {
    pub(crate) inner: Arc<CoreRuntimeInner>,
}

impl CoreHandle {
    /// 创建不保活的运行时引用；回访前必须升级，运行时结束后升级会失败。
    pub fn downgrade(&self) -> CoreWeak {
        CoreWeak {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn scheduler(&self) -> &JobScheduler {
        &self.inner.scheduler
    }

    pub fn task_graph(&self) -> &EngineTaskGraph {
        &self.inner.task_graph
    }

    pub fn task_graph_worker_inventory(&self) -> TaskGraphWorkerInventory {
        self.task_graph().worker_inventory()
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

    pub(crate) fn lock_active_module_order(&self) -> MutexGuard<'_, Vec<String>> {
        lock_poison_recovered(&self.inner.active_module_order)
    }

    // 返回运行中及待重试清理模块的记录顺序；CoreRuntime 关闭时反向遍历它。
    pub(crate) fn active_module_shutdown_order(&self) -> Vec<String> {
        self.lock_active_module_order().clone()
    }

    fn lock_lifecycle_coordinator(&self) -> MutexGuard<'_, LifecycleCoordinator> {
        lock_poison_recovered(&self.inner.lifecycle_coordinator)
    }

    // 首次激活建立注册快照；注册事务先取得同一把锁，避免冻结图遗漏已提交模块。
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
        self.acquire_module_lifecycle_transition_until(module_name, command, None)
    }

    pub(crate) fn acquire_module_lifecycle_transition_until(
        &self,
        module_name: &str,
        command: ModuleLifecycleCommand,
        deadline: Option<Instant>,
    ) -> Result<ModuleLifecycleTransitionPermit, CoreError> {
        self.acquire_module_lifecycle_transition_until_mode(module_name, command, deadline, false)
    }

    pub(crate) fn acquire_module_lifecycle_transition_until_mode(
        &self,
        module_name: &str,
        command: ModuleLifecycleCommand,
        deadline: Option<Instant>,
        allow_expired_owner: bool,
    ) -> Result<ModuleLifecycleTransitionPermit, CoreError> {
        let owner = std::thread::current().id();
        loop {
            let deadline_open_before_lock =
                deadline.map_or(true, |deadline| Instant::now() < deadline);
            if !allow_expired_owner && !deadline_open_before_lock {
                return Err(module_lifecycle_transition_timeout(module_name, command));
            }
            let admission = {
                let mut coordinator = self.lock_lifecycle_coordinator();
                let admission = coordinator.begin(module_name, command, owner)?;
                let deadline_expired = !deadline_open_before_lock
                    || deadline.is_some_and(|deadline| Instant::now() >= deadline);
                if deadline_expired {
                    match admission {
                        ModuleLifecycleTransitionAdmission::Owner(token) if allow_expired_owner => {
                            return Ok(ModuleLifecycleTransitionPermit::Owner(token));
                        }
                        ModuleLifecycleTransitionAdmission::Owner(token) => {
                            let error = module_lifecycle_transition_timeout(module_name, command);
                            coordinator.complete(&token, Err(error.clone()));
                            return Err(error);
                        }
                        ModuleLifecycleTransitionAdmission::Wait(_) => {
                            return Err(module_lifecycle_transition_timeout(module_name, command));
                        }
                    }
                }
                admission
            };
            match admission.resolve_until(deadline) {
                Ok(ModuleLifecycleTransitionPermit::Wait) => {
                    if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                        return Err(module_lifecycle_transition_timeout(module_name, command));
                    }
                }
                Ok(permit) => return Ok(permit),
                Err(()) => return Err(module_lifecycle_transition_timeout(module_name, command)),
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
    }

    // 并发的同名生命周期命令共用协调结果；只有取得令牌的线程执行回调。
    pub(crate) fn run_module_lifecycle_transition<F>(
        &self,
        module_name: &str,
        command: ModuleLifecycleCommand,
        operation: F,
    ) -> Result<(), CoreError>
    where
        F: FnOnce() -> Result<(), CoreError>,
    {
        self.run_module_lifecycle_transition_until(module_name, command, None, operation)
    }

    pub(crate) fn run_module_lifecycle_transition_until<F>(
        &self,
        module_name: &str,
        command: ModuleLifecycleCommand,
        deadline: Option<Instant>,
        operation: F,
    ) -> Result<(), CoreError>
    where
        F: FnOnce() -> Result<(), CoreError>,
    {
        match self.acquire_module_lifecycle_transition_until(module_name, command, deadline)? {
            ModuleLifecycleTransitionPermit::Completed(result) => result,
            ModuleLifecycleTransitionPermit::Owner(token) => {
                let mut owner =
                    ModuleLifecycleTransitionOwner::new(self, token, module_name, command);
                let result = operation();
                owner.complete(result.clone());
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

    // Condvar 原子释放并在唤醒时重新取得服务表锁，供调用方在同一把锁下检查解析状态。
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

    // 每个等待线程只保留一条“等待初始化者”边；若沿已有边回到 waiter，就拒绝闭环等待。
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

    // 懒服务触发模块激活后，启动解析会回访正初始化的槽位；一次性标记允许当前线程通过这次回入。
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

fn module_lifecycle_transition_timeout(
    module_name: &str,
    command: ModuleLifecycleCommand,
) -> CoreError {
    // Admission can expire before the module callback starts, so the stage
    // budget was never entered. Keep zero as the typed metadata marker; the
    // caller-owned absolute deadline still determines whether this branch is
    // reached.
    match command {
        ModuleLifecycleCommand::Activate => CoreError::ModuleReadyTimeout {
            module: module_name.to_owned(),
            budget: Duration::ZERO,
        },
        ModuleLifecycleCommand::Deactivate => CoreError::ModuleCleanupTimeout {
            module: module_name.to_owned(),
            operation: "module_deactivation_transition".to_owned(),
            budget: Duration::ZERO,
            incomplete_entries: 1,
            failed: 0,
            cancelled: 0,
        },
    }
}

// A callback unwind must still publish a terminal receipt for every joined caller.
struct ModuleLifecycleTransitionOwner<'a> {
    handle: &'a CoreHandle,
    token: Option<ModuleLifecycleTransitionToken>,
    module_name: String,
    command: ModuleLifecycleCommand,
}

impl<'a> ModuleLifecycleTransitionOwner<'a> {
    fn new(
        handle: &'a CoreHandle,
        token: ModuleLifecycleTransitionToken,
        module_name: &str,
        command: ModuleLifecycleCommand,
    ) -> Self {
        Self {
            handle,
            token: Some(token),
            module_name: module_name.to_owned(),
            command,
        }
    }

    fn complete(&mut self, result: Result<(), CoreError>) {
        if let Some(token) = self.token.take() {
            self.handle
                .complete_module_lifecycle_transition(&token, result);
        }
    }
}

impl Drop for ModuleLifecycleTransitionOwner<'_> {
    fn drop(&mut self) {
        let error = CoreError::ModuleLifecycleCallbackPanicked {
            module: self.module_name.clone(),
            command: self.command.as_str(),
        };
        self.complete(Err(error));
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
#[path = "tests/core_handle.rs"]
mod tests;
