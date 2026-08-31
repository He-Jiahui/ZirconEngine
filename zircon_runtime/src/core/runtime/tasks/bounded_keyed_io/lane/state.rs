use std::collections::{BTreeSet, HashMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::super::{
    BoundedKeyedIoKey, BoundedKeyedIoTicket, BoundedKeyedIoWork, BoundedKeyedIoWorkDeadline,
    GlobalAdmissionEpoch,
};
use super::BoundedKeyedIoLimits;
use crate::core::runtime::tasks::{JobHandle, JobScheduler, TaskTimerSubscription};

pub(super) type TerminalObserver =
    Arc<dyn Fn(super::BoundedKeyedIoTerminal) + Send + Sync + 'static>;
#[cfg(test)]
pub(super) type BeforeExecuteHook = Arc<dyn Fn() + Send + Sync + 'static>;

pub(crate) struct LaneInner {
    pub(super) scheduler: JobScheduler,
    pub(super) limits: BoundedKeyedIoLimits,
    pub(super) state: Mutex<LaneState>,
    pub(super) changed: Condvar,
    #[cfg(test)]
    pub(super) before_execute: Mutex<Option<BeforeExecuteHook>>,
}

pub(super) struct LaneState {
    pub(super) accepting: bool,
    pub(super) pump_active: bool,
    pub(super) next_ticket_id: u64,
    pub(super) current_epoch: GlobalAdmissionEpoch,
    pub(super) reserved_entries: usize,
    pub(super) retained_bytes: usize,
    pub(super) in_flight: usize,
    pub(super) suspended: HashMap<u64, WorkEntry>,
    pub(super) suspended_order: BTreeSet<(GlobalAdmissionEpoch, u64)>,
    pub(super) active: Option<ActiveEntry>,
    pub(super) queue: VecDeque<WorkEntry>,
    pub(super) active_handles: Vec<JobHandle>,
    pub(super) submitted: u64,
    pub(super) completed: u64,
    pub(super) failed: u64,
    pub(super) cancelled: u64,
    pub(super) superseded: u64,
    pub(super) coalesced: u64,
    pub(super) worker_wall: Duration,
}

pub(super) struct WorkEntry {
    pub(super) key: Option<BoundedKeyedIoKey>,
    pub(super) generation: u64,
    pub(super) epoch: GlobalAdmissionEpoch,
    pub(super) retained_bytes: usize,
    pub(super) enqueued_at: Instant,
    pub(super) deadline: BoundedKeyedIoWorkDeadline,
    pub(super) deadline_subscription: Option<TaskTimerSubscription>,
    pub(super) ticket: BoundedKeyedIoTicket,
    pub(super) terminal_observer: Option<TerminalObserver>,
    pub(super) prerequisites: Box<[FencePrerequisite]>,
    pub(super) work: Option<BoundedKeyedIoWork>,
    pub(super) fence: bool,
}

#[derive(Clone)]
pub(super) struct ActiveEntry {
    pub(super) key: Option<BoundedKeyedIoKey>,
    pub(super) generation: u64,
    pub(super) epoch: GlobalAdmissionEpoch,
    pub(super) enqueued_at: Instant,
    pub(super) ticket: BoundedKeyedIoTicket,
    pub(super) terminal_observer: Option<TerminalObserver>,
    pub(super) fence: bool,
}

#[derive(Clone)]
pub(super) struct FencePrerequisite {
    pub(super) key: Option<BoundedKeyedIoKey>,
    pub(super) generation: u64,
    pub(super) ticket: BoundedKeyedIoTicket,
}

pub(super) struct TerminalNotification {
    pub(super) observer: Option<TerminalObserver>,
    pub(super) terminal: super::BoundedKeyedIoTerminal,
}

impl LaneInner {
    pub(crate) fn lock(&self) -> MutexGuard<'_, LaneState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
