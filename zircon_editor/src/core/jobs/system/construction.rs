use std::sync::{Arc, Condvar, Mutex};

use zircon_runtime::core::runtime::tasks::JobScheduler;

use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::jobs::{EditorJobLimits, EditorJobProgressObserver, EditorJobProgressSource};

use super::super::pump::JobEventPump;
use super::progress_observer::ProgressObserverDispatch;
use super::state::EditorJobSystemState;

#[derive(Clone)]
pub struct EditorJobSystem {
    pub(super) inner: Arc<EditorJobSystemInner>,
}

/// Remaining capacity in the one editor pending-admission queue.
///
/// Domain adapters use this immutable view before constructing request-specific
/// channels or worker payloads. The final submit still rechecks the same queue
/// atomically, so a concurrent admission cannot overcommit the budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorJobAdmissionWindow {
    pending_entries: usize,
    max_pending_entries: usize,
    remaining_entries: usize,
    remaining_estimated_bytes: usize,
    pending_estimated_bytes: usize,
    max_pending_estimated_bytes: usize,
}

impl EditorJobAdmissionWindow {
    pub(super) const fn new(
        pending_entries: usize,
        max_pending_entries: usize,
        remaining_entries: usize,
        remaining_estimated_bytes: usize,
        pending_estimated_bytes: usize,
        max_pending_estimated_bytes: usize,
    ) -> Self {
        Self {
            pending_entries,
            max_pending_entries,
            remaining_entries,
            remaining_estimated_bytes,
            pending_estimated_bytes,
            max_pending_estimated_bytes,
        }
    }

    pub const fn pending_entries(self) -> usize {
        self.pending_entries
    }

    pub const fn max_pending_entries(self) -> usize {
        self.max_pending_entries
    }

    pub const fn remaining_entries(self) -> usize {
        self.remaining_entries
    }

    pub const fn remaining_estimated_bytes(self) -> usize {
        self.remaining_estimated_bytes
    }

    pub const fn pending_estimated_bytes(self) -> usize {
        self.pending_estimated_bytes
    }

    pub const fn max_pending_estimated_bytes(self) -> usize {
        self.max_pending_estimated_bytes
    }
}

pub(super) struct EditorJobSystemInner {
    pub(super) scheduler: JobScheduler,
    pub(super) limits: EditorJobLimits,
    pub(super) event_queue: super::super::pump::JobEventQueue,
    pub(super) event_pump: JobEventPump,
    pub(super) state: Mutex<EditorJobSystemState>,
    pub(super) promotion: Mutex<()>,
    pub(super) state_changed: Condvar,
    pub(super) progress: EditorJobProgressSource,
    pub(super) progress_observer: Option<Arc<dyn EditorJobProgressObserver>>,
    pub(super) progress_observer_dispatch: Mutex<ProgressObserverDispatch>,
}

impl EditorJobSystem {
    pub fn with_scheduler(scheduler: JobScheduler, limits: EditorJobLimits) -> Self {
        Self::with_scheduler_and_bus(scheduler, SharedEditorMessageBus::default(), limits)
    }

    pub fn with_scheduler_and_bus(
        scheduler: JobScheduler,
        bus: SharedEditorMessageBus,
        limits: EditorJobLimits,
    ) -> Self {
        Self::with_scheduler_and_bus_with_progress_observer(scheduler, bus, limits, None)
    }

    pub fn with_scheduler_and_bus_and_progress_observer(
        scheduler: JobScheduler,
        bus: SharedEditorMessageBus,
        limits: EditorJobLimits,
        progress_observer: Arc<dyn EditorJobProgressObserver>,
    ) -> Self {
        Self::with_scheduler_and_bus_with_progress_observer(
            scheduler,
            bus,
            limits,
            Some(progress_observer),
        )
    }

    fn with_scheduler_and_bus_with_progress_observer(
        scheduler: JobScheduler,
        bus: SharedEditorMessageBus,
        limits: EditorJobLimits,
        progress_observer: Option<Arc<dyn EditorJobProgressObserver>>,
    ) -> Self {
        let event_queue = super::super::pump::JobEventQueue::default();
        Self {
            inner: Arc::new(EditorJobSystemInner {
                scheduler,
                limits,
                event_queue: event_queue.clone(),
                event_pump: JobEventPump::new(bus, event_queue),
                state: Mutex::new(EditorJobSystemState::default()),
                promotion: Mutex::new(()),
                state_changed: Condvar::new(),
                progress: EditorJobProgressSource::default(),
                progress_observer,
                progress_observer_dispatch: Mutex::new(ProgressObserverDispatch::default()),
            }),
        }
    }
}
