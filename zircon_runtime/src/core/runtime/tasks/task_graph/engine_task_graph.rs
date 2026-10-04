use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};

use super::super::callback_dispatcher::TaskCallbackDispatcher;
use super::super::{
    JobHandle, JobScheduler, TaskDescriptor, TaskPool, TaskPoolKind, TaskPoolOptions,
    TaskPoolSubmission, TaskPools, TaskTimer,
};
use super::options::{EngineTaskGraphInitError, EngineTaskGraphOptions};
use super::scope::{TaskGraphScope, TaskGraphScopeInner};
use super::{
    TaskGraphAdmissionError, TaskGraphScopeCensus, TaskGraphScopeDescriptor,
    TaskGraphShutdownError, TaskGraphShutdownReport, TaskGraphWorkerDomainInventory,
    TaskGraphWorkerInventory, TaskGraphWorkerShutdownCensus,
};
use crate::core::{CoreError, CoreResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EngineTaskGraphLifecycle {
    Running,
    Closing,
    Stopped,
}

pub(in crate::core::runtime::tasks) struct EngineTaskGraphInner {
    worker_pools: TaskPools,
    callback_dispatcher: TaskCallbackDispatcher,
    owner_identity: Arc<()>,
    state: Mutex<EngineTaskGraphState>,
}

struct EngineTaskGraphState {
    lifecycle: EngineTaskGraphLifecycle,
    lifecycle_timer: Option<TaskTimer>,
    next_scope_id: u64,
    scopes: BTreeMap<u64, Weak<TaskGraphScopeInner>>,
    // Closing owns this fixed set until a final census can replace the strong references.
    shutdown_scopes: Vec<Arc<TaskGraphScopeInner>>,
    stopped_scope_census: Vec<TaskGraphScopeCensus>,
}

/// Runtime-owned graph and sole authority for its physical worker domains.
pub struct EngineTaskGraph {
    inner: Arc<EngineTaskGraphInner>,
}

impl EngineTaskGraph {
    pub fn try_new(options: EngineTaskGraphOptions) -> Result<Self, EngineTaskGraphInitError> {
        let worker_pools = TaskPools::try_from_options(TaskPoolOptions::with_num_threads(
            options.worker_threads(),
        ))?;
        let callback_dispatcher = TaskCallbackDispatcher::new(worker_pools.async_compute().clone());
        Ok(Self {
            inner: Arc::new(EngineTaskGraphInner {
                worker_pools,
                callback_dispatcher,
                owner_identity: Arc::new(()),
                state: Mutex::new(EngineTaskGraphState {
                    lifecycle: EngineTaskGraphLifecycle::Running,
                    lifecycle_timer: None,
                    next_scope_id: 1,
                    scopes: BTreeMap::new(),
                    shutdown_scopes: Vec::new(),
                    stopped_scope_census: Vec::new(),
                }),
            }),
        })
    }

    pub fn create_scope(
        &self,
        descriptor: TaskGraphScopeDescriptor,
    ) -> Result<TaskGraphScope, TaskGraphAdmissionError> {
        let mut state = self.inner.lock_state();
        match state.lifecycle {
            EngineTaskGraphLifecycle::Running => {}
            EngineTaskGraphLifecycle::Closing => {
                return Err(TaskGraphAdmissionError::RuntimeClosing);
            }
            EngineTaskGraphLifecycle::Stopped => {
                return Err(TaskGraphAdmissionError::RuntimeStopped);
            }
        }
        let scope_id = state.next_scope_id;
        state.next_scope_id = state
            .next_scope_id
            .checked_add(1)
            .ok_or(TaskGraphAdmissionError::RuntimeScopeIdExhausted)?;
        let graph = Arc::downgrade(&self.inner);
        let inner = Arc::new(TaskGraphScopeInner::new(
            descriptor,
            graph.clone(),
            scope_id,
        ));
        state.scopes.insert(scope_id, Arc::downgrade(&inner));
        Ok(TaskGraphScope::new(inner, graph))
    }

    /// Binds the scheduler to this graph's lifecycle through a weak reference.
    /// The scheduler retains its pool and has an independent diagnostic store.
    pub fn scheduler(&self, kind: TaskPoolKind) -> JobScheduler {
        self.inner.scheduler_for(kind)
    }

    #[cfg(test)]
    pub(crate) fn task_belongs_to_owner(&self, task: &super::TaskHandle) -> bool {
        task.belongs_to_graph(self.inner.owner_identity())
    }

    pub fn worker_pool(&self) -> &TaskPool {
        self.task_pool(TaskPoolKind::Compute)
    }

    pub fn task_pool(&self, kind: TaskPoolKind) -> &TaskPool {
        self.inner.worker_pools.get(kind)
    }

    /// Lazily starts a timer bound to this graph's asynchronous callback workers.
    /// A spawn failure leaves the graph running and can be retried.
    pub(crate) fn lifecycle_timer(&self) -> CoreResult<TaskTimer> {
        self.lifecycle_timer_with(TaskTimer::new_owned)
    }

    fn lifecycle_timer_with(
        &self,
        create: impl FnOnce(TaskCallbackDispatcher) -> CoreResult<TaskTimer>,
    ) -> CoreResult<TaskTimer> {
        let mut state = self.inner.lock_state();
        if state.lifecycle != EngineTaskGraphLifecycle::Running {
            return Err(CoreError::RuntimeUnavailable);
        }
        if let Some(timer) = state.lifecycle_timer.as_ref() {
            return Ok(timer.clone());
        }
        let timer = create(self.inner.callback_dispatcher.clone())?;
        state.lifecycle_timer = Some(timer.clone());
        Ok(timer)
    }

    /// Reports this graph's three task-pool domains. The owned timer has a
    /// separate shutdown receipt and is not included in the pool worker count.
    /// Process-default and dedicated workers remain outside this graph.
    pub fn worker_inventory(&self) -> TaskGraphWorkerInventory {
        TaskGraphWorkerInventory {
            domains: [
                TaskPoolKind::Io,
                TaskPoolKind::AsyncCompute,
                TaskPoolKind::Compute,
            ]
            .into_iter()
            .map(|kind| {
                let pool = self.task_pool(kind);
                TaskGraphWorkerDomainInventory {
                    kind,
                    worker_count: pool.parallelism(),
                    thread_name: pool.descriptor().thread_name.clone(),
                }
            })
            .collect(),
        }
    }

    /// Closes every scope, requests cooperative cancellation for queued
    /// `CancelOnDrop` work, and waits for every admitted task body to reach a
    /// scope terminal state. A timeout leaves the runtime closing.
    ///
    /// Success proves task-body quiescence and exact joins for the Runtime-owned
    /// worker domains. A timeout leaves the same owner closing so a
    /// later call can continue the transition without recreating workers.
    pub fn shutdown(
        &self,
        deadline: Duration,
    ) -> Result<TaskGraphShutdownReport, TaskGraphShutdownError> {
        let deadline = Instant::now()
            .checked_add(deadline)
            .unwrap_or_else(Instant::now);
        self.shutdown_until(deadline)
    }

    /// Continues shutdown against a caller-owned monotonic deadline.
    ///
    /// Retries must consume the same absolute budget as the enclosing owner;
    /// accepting a relative duration here would restart the budget at every
    /// nested teardown stage.
    pub(crate) fn shutdown_until(
        &self,
        deadline: Instant,
    ) -> Result<TaskGraphShutdownReport, TaskGraphShutdownError> {
        let started_at = Instant::now();
        let scopes = self.inner.begin_shutdown();
        for scope in &scopes {
            scope.close_admission();
        }

        for scope in &scopes {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if !scope.wait_until_quiescent(remaining) {
                return Err(TaskGraphShutdownError {
                    report: self.inner.shutdown_report(started_at.elapsed(), &scopes),
                });
            }
        }

        if let Some(timer) = self.inner.lifecycle_timer_for_shutdown() {
            if !timer.shutdown_until(deadline) {
                return Err(TaskGraphShutdownError {
                    report: self.inner.shutdown_report(started_at.elapsed(), &scopes),
                });
            }
        }

        let _ = self
            .inner
            .worker_pools
            .close_and_join(deadline.saturating_duration_since(Instant::now()));
        let report = self.inner.shutdown_report(started_at.elapsed(), &scopes);
        if report
            .worker_shutdowns
            .iter()
            .any(|workers| !workers.all_joined())
        {
            return Err(TaskGraphShutdownError { report });
        }
        let retained_scopes = self.inner.mark_stopped(report.scopes.clone());
        drop(retained_scopes);
        Ok(report)
    }
}

impl Drop for EngineTaskGraph {
    fn drop(&mut self) {
        // Hosts that can unload code must retain this graph after a shutdown
        // timeout and retry; the final timer owner may block in Drop while
        // joining a callback. This path only closes graph admission.
        for scope in self.inner.begin_shutdown() {
            scope.close_admission();
        }
    }
}

impl EngineTaskGraphInner {
    /// A foreign scope owns admission and cancellation, without reserving a local execution lane.
    pub(super) fn ensure_admission_open(&self) -> Result<(), TaskGraphAdmissionError> {
        match self.lock_state().lifecycle {
            EngineTaskGraphLifecycle::Running => Ok(()),
            EngineTaskGraphLifecycle::Closing => Err(TaskGraphAdmissionError::RuntimeClosing),
            EngineTaskGraphLifecycle::Stopped => Err(TaskGraphAdmissionError::RuntimeStopped),
        }
    }

    pub(in crate::core::runtime::tasks) fn acquire_worker_submission(
        &self,
        kind: TaskPoolKind,
    ) -> Result<TaskPoolSubmission, TaskGraphAdmissionError> {
        let state = self.lock_state();
        match state.lifecycle {
            EngineTaskGraphLifecycle::Running => self
                .worker_pools
                .get(kind)
                .try_acquire_submission()
                .ok_or(TaskGraphAdmissionError::RuntimeClosing),
            EngineTaskGraphLifecycle::Closing => Err(TaskGraphAdmissionError::RuntimeClosing),
            EngineTaskGraphLifecycle::Stopped => Err(TaskGraphAdmissionError::RuntimeStopped),
        }
    }

    pub(super) fn pending_scheduler_completion(
        &self,
        scheduler: &JobScheduler,
        descriptor: TaskDescriptor,
        remaining_dependencies: usize,
    ) -> JobHandle {
        scheduler.pending_graph_task_completion_with_dispatcher(
            descriptor,
            remaining_dependencies,
            self.callback_dispatcher.clone(),
            Arc::downgrade(&self.owner_identity),
        )
    }

    pub(super) fn owner_identity(&self) -> &Arc<()> {
        &self.owner_identity
    }

    pub(super) fn scheduler_for(self: &Arc<Self>, kind: TaskPoolKind) -> JobScheduler {
        JobScheduler::from_pool_with_callback_dispatcher(
            self.worker_pools.get(kind).clone(),
            self.callback_dispatcher.clone(),
        )
        .with_graph_owner(Arc::downgrade(self))
    }

    // Closing 阶段把仍注册的 scope 提升为强引用以供关闭重试继续统计；只有 timer 与 worker 全部 join 后才释放并转为 Stopped。
    fn begin_shutdown(&self) -> Vec<Arc<TaskGraphScopeInner>> {
        let mut state = self.lock_state();
        match state.lifecycle {
            EngineTaskGraphLifecycle::Stopped => return Vec::new(),
            EngineTaskGraphLifecycle::Closing => return state.shutdown_scopes.clone(),
            EngineTaskGraphLifecycle::Running => {}
        }
        state.lifecycle = EngineTaskGraphLifecycle::Closing;
        if let Some(timer) = state.lifecycle_timer.as_ref() {
            timer.close_admission();
        }
        self.worker_pools.close_admission();
        state.scopes.retain(|_, scope| scope.strong_count() > 0);
        state.shutdown_scopes = state.scopes.values().filter_map(Weak::upgrade).collect();
        state.shutdown_scopes.clone()
    }

    fn mark_stopped(
        &self,
        stopped_scope_census: Vec<TaskGraphScopeCensus>,
    ) -> Vec<Arc<TaskGraphScopeInner>> {
        let mut state = self.lock_state();
        state.lifecycle = EngineTaskGraphLifecycle::Stopped;
        state.stopped_scope_census = stopped_scope_census;
        std::mem::take(&mut state.shutdown_scopes)
    }

    pub(super) fn unregister_scope(&self, scope_id: u64) {
        self.lock_state().scopes.remove(&scope_id);
    }

    fn lifecycle_timer_for_shutdown(&self) -> Option<TaskTimer> {
        self.lock_state().lifecycle_timer.clone()
    }

    fn shutdown_report(
        &self,
        elapsed: Duration,
        scopes: &[Arc<TaskGraphScopeInner>],
    ) -> TaskGraphShutdownReport {
        let (lifecycle, stopped_scope_census, timer_started, timer_joined) = {
            let state = self.lock_state();
            (
                state.lifecycle,
                state.stopped_scope_census.clone(),
                state.lifecycle_timer.is_some(),
                state
                    .lifecycle_timer
                    .as_ref()
                    .is_none_or(TaskTimer::is_joined),
            )
        };
        let scope_census = if scopes.is_empty() {
            if lifecycle == EngineTaskGraphLifecycle::Stopped {
                stopped_scope_census
            } else {
                Vec::new()
            }
        } else {
            scopes.iter().map(|scope| scope.census()).collect()
        };
        TaskGraphShutdownReport {
            elapsed,
            scopes: scope_census,
            timer_started,
            timer_joined,
            worker_shutdowns: self
                .worker_pools
                .shutdown_census()
                .into_iter()
                .map(|workers| TaskGraphWorkerShutdownCensus {
                    kind: workers.kind,
                    active_submission_count: workers.active_submission_count,
                    expected_worker_count: workers.expected_worker_count,
                    exited_worker_count: workers.exited_worker_count,
                    joined_worker_count: workers.joined_worker_count,
                    termination_signalled: workers.termination_signalled,
                })
                .collect(),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, EngineTaskGraphState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
#[path = "engine_task_graph/tests/cases.rs"]
mod tests;
