use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use zircon_runtime::core::runtime::tasks::thread_is_join_ready;

use super::{HeadlessController, HeadlessHostError, HeadlessRunReport};
use crate::entry::product_shutdown::{
    ProductFailureLedger, ProductFailureSeverity, ProductHostPhase,
};

pub(super) const STARTUP_CALL_TIMEOUT: Duration = Duration::from_secs(30);
pub(super) const RUNTIME_CALL_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const SHUTDOWN_CALL_TIMEOUT: Duration = Duration::from_secs(3);
const OWNER_EXIT_CHECK_INTERVAL: Duration = Duration::from_millis(1);

#[derive(Debug, Default)]
struct WatchState {
    operation: Option<(&'static str, Instant)>,
    completed: bool,
    result: Option<Result<HeadlessRunReport, HeadlessHostError>>,
    destroy_receipt: Option<Result<(), String>>,
    expired_operation: Option<&'static str>,
    cleanup_pending: bool,
    retry_requested: bool,
    destroy_attempts: u64,
}

#[derive(Debug, Default)]
pub(super) struct OwnerWatch {
    state: Mutex<WatchState>,
    changed: Condvar,
    pub(super) failures: ProductFailureLedger,
}

impl OwnerWatch {
    fn record_terminal(&self, operation: &'static str, error: &HeadlessHostError) {
        let phase = match operation {
            "startup" => ProductHostPhase::Composing,
            "destroy" => ProductHostPhase::DestroyingRuntime,
            _ => ProductHostPhase::Running,
        };
        self.failures.record(
            phase,
            ProductFailureSeverity::Terminal,
            "headless-watchdog",
            error,
        );
    }
    pub(super) fn operation(self: &Arc<Self>, name: &'static str, timeout: Duration) -> Operation {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .operation = Some((name, Instant::now() + timeout));
        self.changed.notify_all();
        Operation(self.clone())
    }

    pub(super) fn record_destroy(&self, result: &Result<(), HeadlessHostError>) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.destroy_receipt = Some(result.as_ref().map(|_| ()).map_err(ToString::to_string));
        state.cleanup_pending = result.is_err();
        state.destroy_attempts = state.destroy_attempts.saturating_add(1);
        self.changed.notify_all();
    }

    pub(super) fn wait_cleanup_retry(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        while !state.retry_requested {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
        state.retry_requested = false;
        state.cleanup_pending = false;
    }

    pub(super) fn notify_cancel(&self) {
        let _state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        self.changed.notify_all();
    }
}

pub(super) struct Operation(Arc<OwnerWatch>);

impl Drop for Operation {
    fn drop(&mut self) {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some((name, deadline)) = state.operation.take() {
            if Instant::now() >= deadline {
                state.expired_operation.get_or_insert(name);
            }
        }
        self.0.changed.notify_all();
    }
}

#[derive(Debug)]
pub(super) struct ManagedOwner {
    thread: Option<JoinHandle<()>>,
    watch: Arc<OwnerWatch>,
}

impl ManagedOwner {
    fn finish_until(&mut self, deadline: Instant) -> bool {
        let mut state = self
            .watch
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        while !state.completed
            || self
                .thread
                .as_ref()
                .is_some_and(|thread| !thread_is_join_ready(thread))
        {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return false;
            };
            if state.cleanup_pending && !state.retry_requested {
                state.retry_requested = true;
                self.watch.changed.notify_all();
            }
            state = self
                .watch
                .changed
                .wait_timeout(state, remaining.min(OWNER_EXIT_CHECK_INTERVAL))
                .unwrap_or_else(|error| error.into_inner())
                .0;
        }
        drop(state);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        true
    }
}

impl Drop for ManagedOwner {
    fn drop(&mut self) {
        if self.thread.is_some() && !self.finish_until(Instant::now()) {
            // Returning from an in-flight DLL call cannot be forced safely. Never detach its owner.
            eprintln!("headless runtime owner remains active at final owner drop");
            std::process::abort();
        }
    }
}

#[derive(Debug, Default)]
struct RetainedOwner {
    active: bool,
    owner: Option<ManagedOwner>,
    destroy_receipt: Option<Result<(), String>>,
}

#[derive(Debug, Default)]
pub(super) struct OwnerRegistry(Mutex<RetainedOwner>);

impl OwnerRegistry {
    pub(super) fn begin(&self) -> Result<(), HeadlessHostError> {
        let mut retained = match self.0.try_lock() {
            Ok(retained) => retained,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err(HeadlessHostError::Startup(
                    "headless owner registry busy".to_owned(),
                ))
            }
        };
        if retained.active {
            return Err(HeadlessHostError::Startup(
                "headless controller already has a session owner".to_owned(),
            ));
        }
        retained.active = true;
        retained.destroy_receipt = None;
        Ok(())
    }

    pub(super) fn complete(&self, watch: &OwnerWatch) {
        let mut retained = self.0.lock().unwrap_or_else(|error| error.into_inner());
        retained.active = false;
        retained.destroy_receipt = watch
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .destroy_receipt
            .clone();
    }
    pub(super) fn destroy_receipt(&self) -> Option<Result<(), String>> {
        let retained = match self.0.try_lock() {
            Ok(retained) => retained,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return None,
        };
        retained
            .owner
            .as_ref()
            .and_then(|owner| {
                owner
                    .watch
                    .state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .destroy_receipt
                    .clone()
            })
            .or_else(|| retained.destroy_receipt.clone())
    }
    pub(super) fn retain(&self, owner: ManagedOwner) {
        let mut retained = self.0.lock().unwrap_or_else(|error| error.into_inner());
        assert!(
            retained.owner.is_none(),
            "headless controller already owns a runtime"
        );
        retained.owner = Some(owner);
    }

    pub(super) fn finish_until(&self, deadline: Instant) -> bool {
        let mut retained = match self.0.try_lock() {
            Ok(retained) => retained,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return false,
        };
        if retained.active && retained.owner.is_none() {
            return false;
        }
        if retained
            .owner
            .as_mut()
            .is_some_and(|owner| !owner.finish_until(deadline))
        {
            return false;
        }
        if let Some(owner) = retained.owner.take() {
            retained.active = false;
            retained.destroy_receipt = owner
                .watch
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .destroy_receipt
                .clone();
        }
        true
    }
}

/// The runtime is created, used, destroyed and dropped on this one managed thread.
/// Only progress and terminal receipts cross the boundary, never RuntimeSession.
pub(super) fn run_owned(
    controller: &HeadlessController,
    operation: impl FnOnce(HeadlessController, Arc<OwnerWatch>) -> Result<HeadlessRunReport, HeadlessHostError>
        + Send
        + 'static,
) -> Result<HeadlessRunReport, HeadlessHostError> {
    run_owned_with_grace(controller, SHUTDOWN_CALL_TIMEOUT, operation)
}

pub(super) fn run_owned_with_grace(
    controller: &HeadlessController,
    cancellation_grace: Duration,
    operation: impl FnOnce(HeadlessController, Arc<OwnerWatch>) -> Result<HeadlessRunReport, HeadlessHostError>
        + Send
        + 'static,
) -> Result<HeadlessRunReport, HeadlessHostError> {
    controller.begin_owner()?;
    let watch = Arc::new(OwnerWatch::default());
    watch
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .operation = Some(("startup", Instant::now() + STARTUP_CALL_TIMEOUT));
    controller.register_watch(&watch);
    let owner_controller = controller.owner_controller();
    let thread_watch = watch.clone();
    let thread = std::thread::Builder::new()
        .name("zircon-headless-session".to_owned())
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                operation(owner_controller, thread_watch.clone())
            }))
            .unwrap_or_else(|_| {
                Err(HeadlessHostError::Runtime(
                    "headless session owner panicked".to_owned(),
                ))
            });
            let mut state = thread_watch
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            state.result = Some(result);
            state.completed = true;
            thread_watch.changed.notify_all();
        })
        .map_err(|error| {
            controller.complete_owner(&watch);
            HeadlessHostError::Startup(error.to_string())
        })?;
    let mut owner = ManagedOwner {
        thread: Some(thread),
        watch: watch.clone(),
    };
    let mut state = watch
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut cancellation_deadline = None;
    loop {
        if state.cleanup_pending {
            let detail = state
                .destroy_receipt
                .as_ref()
                .and_then(|receipt| receipt.as_ref().err())
                .cloned()
                .unwrap_or_else(|| "runtime teardown incomplete".to_owned());
            drop(state);
            controller.retain_owner(owner);
            return Err(HeadlessHostError::RetainedOwner {
                cause: Box::new(HeadlessHostError::Runtime(detail)),
                controller: controller.clone(),
            });
        }
        if let Some(name) = state.expired_operation {
            drop(state);
            let cause = HeadlessHostError::OperationDeadline(name);
            watch.record_terminal(name, &cause);
            controller.cancel();
            controller.retain_owner(owner);
            return Err(HeadlessHostError::RetainedOwner {
                cause: Box::new(cause),
                controller: controller.clone(),
            });
        }
        if state.completed {
            let result = state
                .result
                .take()
                .expect("completed owner publishes terminal receipt");
            drop(state);
            let deadline = controller
                .begin_shutdown()
                .min(Instant::now() + cancellation_grace);
            if !owner.finish_until(deadline) {
                let cause = HeadlessHostError::OperationDeadline("thread-exit");
                watch.record_terminal("destroy", &cause);
                controller.cancel();
                controller.retain_owner(owner);
                return Err(HeadlessHostError::RetainedOwner {
                    cause: Box::new(cause),
                    controller: controller.clone(),
                });
            }
            controller.complete_owner(&watch);
            return result;
        }
        if controller.is_cancelled() && cancellation_deadline.is_none() {
            cancellation_deadline = Some(
                controller
                    .begin_shutdown()
                    .min(Instant::now() + cancellation_grace),
            );
        }
        let failure = if cancellation_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            Some(HeadlessHostError::OperationCancelled(
                state.operation.map(|(name, _)| name).unwrap_or("waiting"),
            ))
        } else if let Some((name, deadline)) = state.operation {
            (Instant::now() >= deadline).then_some(HeadlessHostError::OperationDeadline(name))
        } else {
            None
        };
        if let Some(error) = failure {
            let operation = state.operation.map(|(name, _)| name).unwrap_or("waiting");
            drop(state);
            watch.record_terminal(operation, &error);
            controller.cancel();
            controller.retain_owner(owner);
            return Err(HeadlessHostError::RetainedOwner {
                cause: Box::new(error),
                controller: controller.clone(),
            });
        }
        let deadline = match (
            state.operation.map(|(_, deadline)| deadline),
            cancellation_deadline,
        ) {
            (Some(operation), Some(cancellation)) => Some(operation.min(cancellation)),
            (operation, cancellation) => operation.or(cancellation),
        };
        state = if let Some(deadline) = deadline {
            watch
                .changed
                .wait_timeout(state, deadline.saturating_duration_since(Instant::now()))
                .unwrap_or_else(|error| error.into_inner())
                .0
        } else {
            watch
                .changed
                .wait(state)
                .unwrap_or_else(|error| error.into_inner())
        };
    }
}

#[cfg(test)]
#[path = "tests/managed.rs"]
mod tests;
