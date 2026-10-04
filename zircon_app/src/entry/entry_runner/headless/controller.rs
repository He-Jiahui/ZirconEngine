use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::Instant;

#[derive(Debug, Default)]
struct ControlState {
    cancelled: bool,
    pending_wake: bool,
    shutdown_complete: bool,
    shutdown_deadline: Option<Instant>,
    shutdown_deadline_expired: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HeadlessWait {
    Deadline,
    Woken,
    Cancelled,
}

#[derive(Debug, Default)]
struct ControlSignal {
    state: Mutex<ControlState>,
    changed: Condvar,
    owner_watch: Mutex<Option<Weak<super::managed::OwnerWatch>>>,
}

/// Shared cancellation and wake owner for an independent headless product host.
#[derive(Clone, Debug, Default)]
pub struct HeadlessController(Arc<ControlSignal>, Arc<super::managed::OwnerRegistry>);

impl HeadlessController {
    pub fn cancel(&self) {
        self.begin_shutdown();
        self.0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .cancelled = true;
        self.0.changed.notify_all();
        if let Some(watch) = self
            .0
            .owner_watch
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .and_then(Weak::upgrade)
        {
            watch.notify_cancel();
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .cancelled
    }

    /// Starts the process shutdown budget once; later phases reuse the same deadline.
    pub fn begin_shutdown(&self) -> Instant {
        self.begin_shutdown_until(Instant::now() + super::managed::SHUTDOWN_CALL_TIMEOUT)
    }

    pub(super) fn begin_shutdown_until(&self, deadline: Instant) -> Instant {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *state.shutdown_deadline.get_or_insert(deadline)
    }

    /// Reaps the retained runtime owner after cancellation or a call deadline.
    /// A pending destroy failure schedules retries on its original owner thread until the
    /// supplied deadline, preserving the owner when the deadline expires.
    /// False means the owner is active or another caller is reaping it; the DLL stays loaded.
    pub fn finish_runtime_until(&self, deadline: Instant) -> bool {
        self.1.finish_until(deadline)
    }

    /// None means no destroy result is available yet, including while another caller reaps.
    pub fn runtime_destroy_receipt(&self) -> Option<Result<(), String>> {
        self.1.destroy_receipt()
    }

    /// Publishes the terminal shutdown outcome, retaining any deadline failure.
    pub fn mark_shutdown_complete(&self) {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if state
            .shutdown_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            state.shutdown_deadline_expired = true;
        }
        state.shutdown_complete = true;
        drop(state);
        self.0.changed.notify_all();
    }

    pub fn wait_shutdown_until(&self, deadline: Instant) -> bool {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let deadline = state
            .shutdown_deadline
            .map_or(deadline, |shared| shared.min(deadline));
        while !state.shutdown_complete {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                state.shutdown_deadline_expired = true;
                return false;
            };
            state = self
                .0
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|error| error.into_inner())
                .0;
        }
        true
    }

    pub fn shutdown_deadline_expired(&self) -> bool {
        self.0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .shutdown_deadline_expired
    }

    pub(super) fn owner_controller(&self) -> Self {
        Self(self.0.clone(), Arc::default())
    }

    pub(super) fn register_watch(&self, watch: &Arc<super::managed::OwnerWatch>) {
        *self
            .0
            .owner_watch
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(Arc::downgrade(watch));
    }

    pub(super) fn retain_owner(&self, owner: super::managed::ManagedOwner) {
        self.1.retain(owner);
    }

    pub(super) fn begin_owner(&self) -> Result<(), super::HeadlessHostError> {
        self.1.begin()
    }

    pub(super) fn complete_owner(&self, watch: &super::managed::OwnerWatch) {
        self.1.complete(watch);
    }

    pub(crate) fn wake(&self) {
        self.0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .pending_wake = true;
        self.0.changed.notify_all();
    }

    pub(super) fn wait_event(&self, deadline: Instant) -> HeadlessWait {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        loop {
            if state.cancelled {
                return HeadlessWait::Cancelled;
            }
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                state.pending_wake = false;
                return HeadlessWait::Deadline;
            };
            if remaining.is_zero() {
                state.pending_wake = false;
                return HeadlessWait::Deadline;
            }
            if std::mem::take(&mut state.pending_wake) {
                return HeadlessWait::Woken;
            }
            state = self
                .0
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|error| error.into_inner())
                .0;
        }
    }
}
