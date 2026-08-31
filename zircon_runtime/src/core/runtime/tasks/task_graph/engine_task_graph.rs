use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};

use super::super::callback_dispatcher::TaskCallbackDispatcher;
use super::super::{JobHandle, JobScheduler, TaskPool, TaskPoolDescriptor, TaskPoolSubmission};
use super::options::{EngineTaskGraphInitError, EngineTaskGraphOptions};
use super::scope::{TaskGraphScope, TaskGraphScopeInner};
use super::{
    TaskGraphAdmissionError, TaskGraphScopeDescriptor, TaskGraphShutdownError,
    TaskGraphShutdownReport, TaskGraphWorkerInventory, TaskGraphWorkerShutdownCensus,
};

const TASK_GRAPH_WORKER_THREAD_NAME: &str = "zircon-taskgraph-worker";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EngineTaskGraphLifecycle {
    Running,
    Closing,
    Stopped,
}

pub(super) struct EngineTaskGraphInner {
    worker_pool: TaskPool,
    callback_dispatcher: TaskCallbackDispatcher,
    state: Mutex<EngineTaskGraphState>,
}

struct EngineTaskGraphState {
    lifecycle: EngineTaskGraphLifecycle,
    next_scope_id: u64,
    scopes: BTreeMap<u64, Weak<TaskGraphScopeInner>>,
}

/// Runtime-owned graph and sole physical worker-set owner.
pub struct EngineTaskGraph {
    inner: Arc<EngineTaskGraphInner>,
}

impl EngineTaskGraph {
    pub fn try_new(options: EngineTaskGraphOptions) -> Result<Self, EngineTaskGraphInitError> {
        let worker_pool = TaskPool::try_new(
            TaskPoolDescriptor::compute()
                .with_worker_threads(options.worker_threads())
                .with_thread_name(TASK_GRAPH_WORKER_THREAD_NAME),
        )?;
        let callback_dispatcher = TaskCallbackDispatcher::new(worker_pool.clone());
        Ok(Self {
            inner: Arc::new(EngineTaskGraphInner {
                worker_pool,
                callback_dispatcher,
                state: Mutex::new(EngineTaskGraphState {
                    lifecycle: EngineTaskGraphLifecycle::Running,
                    next_scope_id: 1,
                    scopes: BTreeMap::new(),
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

    pub fn worker_pool(&self) -> &TaskPool {
        &self.inner.worker_pool
    }

    /// Reports the worker set constructed directly by this runtime.
    ///
    /// The inventory intentionally excludes process-default, timer, and
    /// dedicated worker owners until each adopts the same lifecycle contract.
    pub fn worker_inventory(&self) -> TaskGraphWorkerInventory {
        TaskGraphWorkerInventory {
            worker_set_count: 1,
            worker_count: self.worker_pool().parallelism(),
            thread_name: self.worker_pool().descriptor().thread_name.clone(),
        }
    }

    /// Closes every scope, requests cooperative cancellation for queued
    /// `CancelOnDrop` work, and waits for every admitted task body to reach a
    /// scope terminal state. A timeout leaves the runtime closing.
    ///
    /// Success proves task-body quiescence and exact joins for the Runtime-owned
    /// worker set. A timeout leaves the same owner closing so a
    /// later call can continue the transition without recreating workers.
    pub fn shutdown(
        &self,
        deadline: Duration,
    ) -> Result<TaskGraphShutdownReport, TaskGraphShutdownError> {
        let started_at = Instant::now();
        let scopes = self.inner.begin_shutdown();
        for scope in &scopes {
            scope.close_admission();
        }

        for scope in &scopes {
            let remaining = deadline.saturating_sub(started_at.elapsed());
            if !scope.wait_until_quiescent(remaining) {
                return Err(TaskGraphShutdownError {
                    report: self.inner.shutdown_report(started_at.elapsed(), &scopes),
                });
            }
        }

        let remaining = deadline.saturating_sub(started_at.elapsed());
        let _ = self.inner.worker_pool.close_and_join(remaining);
        let report = self.inner.shutdown_report(started_at.elapsed(), &scopes);
        if !report.worker_shutdown.all_joined() {
            return Err(TaskGraphShutdownError { report });
        }
        self.inner.mark_stopped();
        Ok(report)
    }
}

impl Drop for EngineTaskGraph {
    fn drop(&mut self) {
        // Hosts that can unload code must call `shutdown` and treat a timeout
        // as teardown-incomplete. Drop only prevents further scoped admission.
        for scope in self.inner.begin_shutdown() {
            scope.close_admission();
        }
    }
}

impl EngineTaskGraphInner {
    pub(super) fn acquire_worker_submission(
        &self,
    ) -> Result<TaskPoolSubmission, TaskGraphAdmissionError> {
        let state = self.lock_state();
        match state.lifecycle {
            EngineTaskGraphLifecycle::Running => self
                .worker_pool
                .try_acquire_submission()
                .ok_or(TaskGraphAdmissionError::RuntimeClosing),
            EngineTaskGraphLifecycle::Closing => Err(TaskGraphAdmissionError::RuntimeClosing),
            EngineTaskGraphLifecycle::Stopped => Err(TaskGraphAdmissionError::RuntimeStopped),
        }
    }

    pub(super) fn pending_completion(&self) -> JobHandle {
        JobHandle::pending_with_callback_dispatcher(0, self.callback_dispatcher.clone())
    }

    pub(super) fn shares_worker_owner_with(&self, scheduler: &JobScheduler) -> bool {
        scheduler.shares_execution_owner_with(&self.worker_pool)
    }

    fn begin_shutdown(&self) -> Vec<Arc<TaskGraphScopeInner>> {
        let mut state = self.lock_state();
        if state.lifecycle == EngineTaskGraphLifecycle::Stopped {
            return Vec::new();
        }
        state.lifecycle = EngineTaskGraphLifecycle::Closing;
        self.worker_pool.close_admission();
        state.scopes.retain(|_, scope| scope.strong_count() > 0);
        state.scopes.values().filter_map(Weak::upgrade).collect()
    }

    fn mark_stopped(&self) {
        self.lock_state().lifecycle = EngineTaskGraphLifecycle::Stopped;
    }

    pub(super) fn unregister_scope(&self, scope_id: u64) {
        self.lock_state().scopes.remove(&scope_id);
    }

    fn shutdown_report(
        &self,
        elapsed: Duration,
        scopes: &[Arc<TaskGraphScopeInner>],
    ) -> TaskGraphShutdownReport {
        let worker = self.worker_pool.shutdown_census();
        TaskGraphShutdownReport {
            elapsed,
            scopes: scopes.iter().map(|scope| scope.census()).collect(),
            worker_shutdown: TaskGraphWorkerShutdownCensus {
                active_submission_count: worker.active_submission_count,
                expected_worker_count: worker.expected_worker_count,
                exited_worker_count: worker.exited_worker_count,
                joined_worker_count: worker.joined_worker_count,
                termination_signalled: worker.termination_signalled,
            },
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, EngineTaskGraphState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::super::EngineTaskGraphOptions;
    use super::EngineTaskGraph;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    #[test]
    fn worker_inventory_reports_one_shared_set_for_exact_global_budgets() {
        for worker_count in [1, 2, 7] {
            let graph =
                EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(worker_count))
                    .expect("task graph should create one shared worker set");

            let inventory = graph.worker_inventory();

            assert_eq!(inventory.worker_count, worker_count);
            assert_eq!(inventory.worker_set_count, 1);
            assert_eq!(inventory.thread_name, "zircon-taskgraph-worker");
        }
    }

    #[test]
    fn dropping_empty_scopes_retires_graph_registration_immediately() {
        let graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
            .expect("task graph should create its worker budget");

        for index in 0..1_024 {
            let scope = graph
                .create_scope(super::TaskGraphScopeDescriptor::new(format!(
                    "scope-{index}"
                )))
                .expect("running task graph should admit a scope");
            drop(scope);
        }

        assert_eq!(graph.inner.lock_state().scopes.len(), 0);
    }

    #[test]
    fn shutdown_joins_owned_workers_even_when_pool_handles_are_retained() {
        let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("task graph should create its worker budget");
        let retained_pool_handle = runtime.worker_pool().clone();

        let report = runtime
            .shutdown(Duration::from_secs(2))
            .expect("an idle runtime should join every owned worker");

        assert!(report.worker_shutdown.all_joined());
        assert!(report.worker_shutdown.termination_signalled);
        assert_eq!(report.worker_shutdown.expected_worker_count, 3);
        assert_eq!(report.worker_shutdown.exited_worker_count, 3);
        assert_eq!(report.worker_shutdown.joined_worker_count, 3);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            retained_pool_handle.spawn(|| {});
        }))
        .is_err());

        let repeated = runtime
            .shutdown(Duration::from_secs(2))
            .expect("repeated shutdown should preserve the joined receipt");
        assert!(repeated.worker_shutdown.all_joined());
        drop(retained_pool_handle);
    }

    #[test]
    fn shutdown_timeout_keeps_unjoined_workers_visible_and_retryable() {
        let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("task graph should create its worker budget");
        let worker_pool = runtime.worker_pool().clone();
        let (started_sender, started_receiver) = mpsc::sync_channel(1);
        let (release_sender, release_receiver) = mpsc::sync_channel(1);
        worker_pool.spawn(move || {
            started_sender
                .send(())
                .expect("test owner should observe worker start");
            release_receiver
                .recv()
                .expect("test owner should release blocked worker");
        });
        started_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("unscoped worker task should start");

        let timeout = runtime
            .shutdown(Duration::ZERO)
            .expect_err("a blocked unscoped worker must prevent a joined receipt");
        let workers = &timeout.report.worker_shutdown;
        assert_eq!(workers.active_submission_count, 1);
        assert!(!workers.termination_signalled);
        assert!(workers.joined_worker_count < workers.expected_worker_count);
        assert!(timeout.report.has_in_flight_work());

        release_sender
            .send(())
            .expect("blocked worker should still be owned by the closing runtime");
        let retry = runtime
            .shutdown(Duration::from_secs(2))
            .expect("shutdown retry should join the released worker");
        assert!(retry.worker_shutdown.termination_signalled);
        assert_eq!(retry.worker_shutdown.active_submission_count, 0);
        assert!(retry.worker_shutdown.all_joined());
    }

    #[test]
    fn shutdown_waits_for_accepted_terminal_observers_before_worker_quit() {
        let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
            .expect("task graph should create its worker budget");
        let scope = runtime
            .create_scope(super::TaskGraphScopeDescriptor::new("terminal-observer"))
            .expect("running task graph should admit a scope");
        let scheduler =
            crate::core::runtime::tasks::JobScheduler::from_pool(runtime.worker_pool().clone());
        let scheduled = scope
            .schedule(
                &scheduler,
                crate::core::runtime::tasks::TaskDescriptor::new(
                    crate::core::runtime::tasks::TaskId::new(1),
                    crate::core::runtime::tasks::TaskPoolKind::Compute,
                    "terminal-observer",
                ),
                |_| {},
            )
            .expect("scope should accept scheduled work");
        let (observer_started_tx, observer_started_rx) = mpsc::sync_channel(1);
        let (observer_release_tx, observer_release_rx) = mpsc::sync_channel(1);
        scheduled.on_terminal(move || {
            observer_started_tx
                .send(())
                .expect("test owner should observe terminal callback start");
            observer_release_rx
                .recv()
                .expect("test owner should release terminal callback");
        });
        scheduled.wait();
        observer_started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("accepted terminal callback should reach the worker");

        let timeout = runtime
            .shutdown(Duration::ZERO)
            .expect_err("active terminal callback must keep its execution lease visible");
        assert_eq!(timeout.report.worker_shutdown.active_submission_count, 1);
        assert!(!timeout.report.worker_shutdown.termination_signalled);

        observer_release_tx
            .send(())
            .expect("terminal callback should remain owned after timeout");
        let retry = runtime
            .shutdown(Duration::from_secs(1))
            .expect("shutdown retry should wait for callback completion and join");
        assert_eq!(retry.worker_shutdown.active_submission_count, 0);
        assert!(retry.worker_shutdown.all_joined());
    }

    #[test]
    fn shutdown_from_owned_worker_returns_incomplete_without_self_joining() {
        let runtime = Arc::new(
            EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
                .expect("task graph should create its worker budget"),
        );
        let worker_runtime = Arc::clone(&runtime);
        let (result_sender, result_receiver) = mpsc::sync_channel(1);
        runtime.worker_pool().spawn(move || {
            let error = worker_runtime
                .shutdown(Duration::from_secs(30))
                .expect_err("a worker cannot publish a receipt that joins itself");
            result_sender
                .send(error.report)
                .expect("test owner should receive the incomplete receipt");
        });

        let report = result_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("worker-side shutdown must not wait for its own deadline");
        assert!(!report.worker_shutdown.all_joined());

        let retry = runtime
            .shutdown(Duration::from_secs(2))
            .expect("external retry should join the worker after its task returns");
        assert!(retry.worker_shutdown.all_joined());
    }
}
