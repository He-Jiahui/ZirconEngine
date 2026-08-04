use std::collections::{HashMap, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::{
    BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority,
    BoundedKeyedIoDiagnostics, BoundedKeyedIoFailure, BoundedKeyedIoFence, BoundedKeyedIoTerminal,
    BoundedKeyedIoTicket, BoundedKeyedIoWork, BoundedKeyedIoWorkDeadline, GlobalAdmissionEpoch,
};
use crate::core::runtime::tasks::{JobHandle, JobScheduler, TaskTimer, TaskTimerSubscription};

mod coalescing;
mod fence_prerequisites;
mod shutdown;

use coalescing::{coalesce_queued_generation, insert_ordered};
use fence_prerequisites::{
    capture_fence_prerequisites, fence_prerequisite_failure, plan_fence_prerequisites,
    release_fence_pins,
};
use shutdown::diagnostics_for_state;
pub use shutdown::BoundedKeyedIoShutdownGuard;

type TerminalObserver = Arc<dyn Fn(BoundedKeyedIoTerminal) + Send + Sync + 'static>;
#[cfg(test)]
type BeforeExecuteHook = Arc<dyn Fn() + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundedKeyedIoLimits {
    pub max_entries: usize,
    pub max_retained_bytes: usize,
}

impl BoundedKeyedIoLimits {
    pub const fn new(max_entries: usize, max_retained_bytes: usize) -> Self {
        Self {
            max_entries,
            max_retained_bytes,
        }
    }
}

#[derive(Clone)]
pub struct BoundedKeyedIoLane {
    inner: Arc<LaneInner>,
}

pub(crate) struct LaneInner {
    scheduler: JobScheduler,
    limits: BoundedKeyedIoLimits,
    state: Mutex<LaneState>,
    changed: Condvar,
    #[cfg(test)]
    before_execute: Mutex<Option<BeforeExecuteHook>>,
}

struct LaneState {
    accepting: bool,
    pump_active: bool,
    next_ticket_id: u64,
    current_epoch: GlobalAdmissionEpoch,
    reserved_entries: usize,
    retained_bytes: usize,
    in_flight: usize,
    suspended: HashMap<u64, WorkEntry>,
    active: Option<ActiveEntry>,
    queue: VecDeque<WorkEntry>,
    active_handles: Vec<JobHandle>,
    submitted: u64,
    completed: u64,
    failed: u64,
    cancelled: u64,
    superseded: u64,
    coalesced: u64,
    worker_wall: Duration,
}

struct WorkEntry {
    key: Option<Arc<str>>,
    generation: u64,
    epoch: GlobalAdmissionEpoch,
    retained_bytes: usize,
    enqueued_at: Instant,
    deadline: BoundedKeyedIoWorkDeadline,
    deadline_subscription: Option<TaskTimerSubscription>,
    ticket: BoundedKeyedIoTicket,
    terminal_observer: Option<TerminalObserver>,
    prerequisites: Box<[FencePrerequisite]>,
    work: Option<BoundedKeyedIoWork>,
    fence: bool,
}

#[derive(Clone)]
struct ActiveEntry {
    key: Option<Arc<str>>,
    generation: u64,
    epoch: GlobalAdmissionEpoch,
    enqueued_at: Instant,
    ticket: BoundedKeyedIoTicket,
    terminal_observer: Option<TerminalObserver>,
    fence: bool,
}

#[derive(Clone)]
struct FencePrerequisite {
    key: Option<Arc<str>>,
    generation: u64,
    ticket: BoundedKeyedIoTicket,
}

struct TerminalNotification {
    observer: Option<TerminalObserver>,
    terminal: BoundedKeyedIoTerminal,
}

impl BoundedKeyedIoLane {
    pub fn new(limits: BoundedKeyedIoLimits, scheduler: JobScheduler) -> Self {
        Self {
            inner: Arc::new(LaneInner {
                scheduler,
                limits,
                state: Mutex::new(LaneState {
                    accepting: true,
                    pump_active: false,
                    next_ticket_id: 1,
                    current_epoch: GlobalAdmissionEpoch::initial(),
                    reserved_entries: 0,
                    retained_bytes: 0,
                    in_flight: 0,
                    suspended: HashMap::new(),
                    active: None,
                    queue: VecDeque::new(),
                    active_handles: Vec::new(),
                    submitted: 0,
                    completed: 0,
                    failed: 0,
                    cancelled: 0,
                    superseded: 0,
                    coalesced: 0,
                    worker_wall: Duration::ZERO,
                }),
                changed: Condvar::new(),
                #[cfg(test)]
                before_execute: Mutex::new(None),
            }),
        }
    }

    #[cfg(test)]
    pub(super) fn set_before_execute_hook(&self, hook: impl Fn() + Send + Sync + 'static) {
        *self
            .inner
            .before_execute
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::new(hook));
    }

    pub fn try_admit(
        &self,
        key: impl Into<Arc<str>>,
        generation: u64,
        retained_bytes: usize,
        deadline: BoundedKeyedIoWorkDeadline,
        work: BoundedKeyedIoWork,
    ) -> Result<BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError> {
        let (id, epoch, ticket) = {
            let mut state = self.inner.lock();
            reserve(&self.inner, &mut state, retained_bytes)?;
            let id = take_ticket_id(&mut state);
            let ticket = BoundedKeyedIoTicket::pending(id, generation, false);
            let epoch = state.current_epoch;
            state.suspended.insert(
                id,
                WorkEntry {
                    key: Some(key.into()),
                    generation,
                    epoch,
                    retained_bytes,
                    enqueued_at: Instant::now(),
                    deadline,
                    deadline_subscription: None,
                    ticket: ticket.clone(),
                    terminal_observer: None,
                    prerequisites: Box::new([]),
                    work: Some(work),
                    fence: false,
                },
            );
            (id, epoch, ticket)
        };

        if let Some(deadline) = deadline.instant() {
            let timer = TaskTimer::process_default().map_err(|_| {
                LaneInner::rollback_admission(&self.inner, id);
                BoundedKeyedIoAdmissionError::DeadlineTimerUnavailable
            })?;
            let weak_lane = Arc::downgrade(&self.inner);
            let subscription = timer
                .schedule_at(deadline, move || {
                    if let Some(lane) = weak_lane.upgrade() {
                        LaneInner::expire_before_start(&lane, id);
                    }
                })
                .map_err(|_| {
                    LaneInner::rollback_admission(&self.inner, id);
                    BoundedKeyedIoAdmissionError::DeadlineTimerUnavailable
                })?;
            if let Some(entry) = self.inner.lock().suspended.get_mut(&id) {
                entry.deadline_subscription = Some(subscription);
            }
        }

        Ok(BoundedKeyedIoAdmission {
            lane: Arc::clone(&self.inner),
            ticket_id: id,
            epoch,
            armed: true,
            ticket: ticket.clone(),
            cancel_authority: BoundedKeyedIoCancelAuthority::new(id),
        })
    }

    pub fn submit_fence(
        &self,
        retained_bytes: usize,
        deadline: BoundedKeyedIoWorkDeadline,
        work: BoundedKeyedIoWork,
    ) -> Result<BoundedKeyedIoFence, BoundedKeyedIoAdmissionError> {
        let (epoch, ticket, start_pump) = {
            let mut state = self.inner.lock();
            let epoch = state.current_epoch;
            let prerequisite_plan = plan_fence_prerequisites(&state, epoch)
                .ok_or(BoundedKeyedIoAdmissionError::RetainedBytesOverflow)?;
            let prerequisite_bytes = prerequisite_plan
                .retained_bytes()
                .ok_or(BoundedKeyedIoAdmissionError::RetainedBytesOverflow)?;
            let charged_retained_bytes = retained_bytes
                .checked_add(prerequisite_bytes)
                .ok_or(BoundedKeyedIoAdmissionError::RetainedBytesOverflow)?;
            reserve(&self.inner, &mut state, charged_retained_bytes)?;
            let id = take_ticket_id(&mut state);
            let deadline_subscription = if let Some(deadline) = deadline.instant() {
                let timer = TaskTimer::process_default().map_err(|_| {
                    release_reservation(&mut state, charged_retained_bytes);
                    state.submitted = state.submitted.saturating_sub(1);
                    BoundedKeyedIoAdmissionError::DeadlineTimerUnavailable
                })?;
                let weak_lane = Arc::downgrade(&self.inner);
                Some(
                    timer
                        .schedule_at(deadline, move || {
                            if let Some(lane) = weak_lane.upgrade() {
                                LaneInner::expire_before_start(&lane, id);
                            }
                        })
                        .map_err(|_| {
                            release_reservation(&mut state, charged_retained_bytes);
                            state.submitted = state.submitted.saturating_sub(1);
                            BoundedKeyedIoAdmissionError::DeadlineTimerUnavailable
                        })?,
                )
            } else {
                None
            };
            state.current_epoch = GlobalAdmissionEpoch(epoch.0.saturating_add(1));
            let prerequisites = capture_fence_prerequisites(&state, prerequisite_plan);
            for prerequisite in &prerequisites {
                prerequisite.ticket.pin_to_fence();
            }
            let ticket = BoundedKeyedIoTicket::pending(id, 0, true);
            state.queue.push_back(WorkEntry {
                key: None,
                generation: 0,
                epoch,
                retained_bytes: charged_retained_bytes,
                enqueued_at: Instant::now(),
                deadline,
                deadline_subscription,
                ticket: ticket.clone(),
                terminal_observer: None,
                prerequisites: prerequisites.into_boxed_slice(),
                work: Some(work),
                fence: true,
            });
            let start_pump = mark_pump_needed(&mut state);
            (epoch, ticket, start_pump)
        };
        if start_pump {
            LaneInner::start_pump(&self.inner);
        }
        Ok(BoundedKeyedIoFence::new(epoch, ticket))
    }

    pub fn diagnostics(&self) -> BoundedKeyedIoDiagnostics {
        let state = self.inner.lock();
        diagnostics_for_state(&state)
    }

    pub fn shutdown(&self) -> BoundedKeyedIoShutdownGuard {
        let (notifications, start_pump) = {
            let mut state = self.inner.lock();
            state.accepting = false;
            let mut notifications = Vec::new();

            for (_, entry) in std::mem::take(&mut state.suspended) {
                if entry.fence || entry.ticket.fence_pinned() {
                    insert_ordered(&mut state.queue, entry);
                } else {
                    finish_pre_start_entry(
                        &mut state,
                        entry,
                        BoundedKeyedIoTerminal::Shutdown,
                        &mut notifications,
                    );
                }
            }

            let mut retained = VecDeque::new();
            while let Some(entry) = state.queue.pop_front() {
                if entry.fence || entry.ticket.fence_pinned() {
                    retained.push_back(entry);
                } else {
                    finish_pre_start_entry(
                        &mut state,
                        entry,
                        BoundedKeyedIoTerminal::Shutdown,
                        &mut notifications,
                    );
                }
            }
            state.queue = retained;
            let start_pump = mark_pump_needed(&mut state);
            (notifications, start_pump)
        };
        self.inner.changed.notify_all();
        notify_observers(notifications);
        if start_pump {
            LaneInner::start_pump(&self.inner);
        }
        BoundedKeyedIoShutdownGuard {
            lane: Arc::clone(&self.inner),
        }
    }
}

impl LaneInner {
    pub(crate) fn observe_terminal(
        lane: &Arc<Self>,
        ticket_id: u64,
        ticket: &BoundedKeyedIoTicket,
        observer: TerminalObserver,
    ) {
        let terminal = {
            let mut state = lane.lock();
            let registered = if let Some(entry) = state.suspended.get_mut(&ticket_id) {
                entry.terminal_observer = Some(Arc::clone(&observer));
                true
            } else if let Some(entry) = state
                .queue
                .iter_mut()
                .find(|entry| entry.ticket.id() == ticket_id)
            {
                entry.terminal_observer = Some(Arc::clone(&observer));
                true
            } else if let Some(active) = state
                .active
                .as_mut()
                .filter(|active| active.ticket.id() == ticket_id)
            {
                active.terminal_observer = Some(Arc::clone(&observer));
                true
            } else {
                false
            };
            (!registered).then(|| ticket.terminal()).flatten()
        };
        if let Some(terminal) = terminal {
            notify_observer(Some(observer), terminal);
        }
    }

    pub(crate) fn activate(lane: &Arc<Self>, ticket_id: u64) {
        let (notifications, start_pump) = {
            let mut state = lane.lock();
            let Some(entry) = state.suspended.remove(&ticket_id) else {
                return;
            };
            let mut notifications = Vec::new();
            if let Some(terminal) = entry.ticket.terminal() {
                finish_pre_start_entry(&mut state, entry, terminal, &mut notifications);
            } else if !state.accepting {
                finish_pre_start_entry(
                    &mut state,
                    entry,
                    BoundedKeyedIoTerminal::Shutdown,
                    &mut notifications,
                );
            } else if coalesce_queued_generation(&mut state, &entry, &mut notifications) {
                insert_ordered(&mut state.queue, entry);
            }
            let start_pump = mark_pump_needed(&mut state);
            (notifications, start_pump)
        };
        lane.changed.notify_all();
        notify_observers(notifications);
        if start_pump {
            Self::start_pump(lane);
        }
    }

    pub(crate) fn release_unactivated(lane: &Arc<Self>, ticket_id: u64) {
        let (notifications, start_pump) = {
            let mut state = lane.lock();
            let Some(entry) = state.suspended.remove(&ticket_id) else {
                return;
            };
            let mut notifications = Vec::new();
            if entry.ticket.fence_pinned() {
                insert_ordered(&mut state.queue, entry);
            } else {
                finish_pre_start_entry(
                    &mut state,
                    entry,
                    BoundedKeyedIoTerminal::CancelledBeforeStart,
                    &mut notifications,
                );
            }
            let start_pump = mark_pump_needed(&mut state);
            (notifications, start_pump)
        };
        lane.changed.notify_all();
        notify_observers(notifications);
        if start_pump {
            Self::start_pump(lane);
        }
    }

    fn rollback_admission(lane: &Arc<Self>, ticket_id: u64) {
        let mut state = lane.lock();
        if let Some(entry) = state.suspended.remove(&ticket_id) {
            release_reservation(&mut state, entry.retained_bytes);
            state.submitted = state.submitted.saturating_sub(1);
        }
        drop(state);
        lane.changed.notify_all();
    }

    fn expire_before_start(lane: &Arc<Self>, ticket_id: u64) {
        let (notification, start_pump) = {
            let mut state = lane.lock();
            let entry = state.suspended.remove(&ticket_id).or_else(|| {
                let index = state
                    .queue
                    .iter()
                    .position(|entry| entry.ticket.id() == ticket_id)?;
                state.queue.remove(index)
            });
            if let Some(entry) = entry {
                release_fence_pins(&entry);
                let terminal = if entry
                    .ticket
                    .mark_terminal_before_start(BoundedKeyedIoTerminal::DeadlineBeforeStart)
                {
                    BoundedKeyedIoTerminal::DeadlineBeforeStart
                } else {
                    entry
                        .ticket
                        .terminal()
                        .unwrap_or(BoundedKeyedIoTerminal::Shutdown)
                };
                release_reservation(&mut state, entry.retained_bytes);
                state.cancelled = state.cancelled.saturating_add(1);
                let notification = TerminalNotification {
                    observer: entry.terminal_observer,
                    terminal,
                };
                let start_pump = mark_pump_needed(&mut state);
                (Some(notification), start_pump)
            } else {
                (None, false)
            }
        };
        lane.changed.notify_all();
        if let Some(notification) = notification {
            notify_observers(vec![notification]);
        }
        if start_pump {
            Self::start_pump(lane);
        }
    }

    fn start_pump(lane: &Arc<Self>) {
        let lane_for_work = Arc::clone(lane);
        let handle = lane.scheduler.schedule(move || lane_for_work.pump());
        lane.lock().active_handles.push(handle.clone());
        lane.changed.notify_all();
        let weak_lane = Arc::downgrade(lane);
        handle.on_terminal(move || {
            let Some(lane) = weak_lane.upgrade() else {
                return;
            };
            lane.lock()
                .active_handles
                .retain(|handle| !handle.is_complete());
            lane.changed.notify_all();
        });
    }

    fn pump(&self) {
        loop {
            let Some(mut entry) = self.next_entry() else {
                return;
            };
            if entry.ticket.terminal().is_some() {
                self.finish_skipped(entry);
                continue;
            }
            #[cfg(test)]
            if let Some(hook) = self
                .before_execute
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take()
            {
                hook();
            }

            let started = Instant::now();
            let result = catch_unwind(AssertUnwindSafe(|| {
                entry
                    .work
                    .take()
                    .expect("activated bounded I/O entry must own work")()
            }));
            let elapsed = started.elapsed();
            let terminal = match result {
                Ok(Ok(())) => BoundedKeyedIoTerminal::Succeeded,
                Ok(Err(error)) => BoundedKeyedIoTerminal::Failed(error),
                Err(_) => {
                    BoundedKeyedIoTerminal::Failed(BoundedKeyedIoFailure::new("work_panicked"))
                }
            };
            entry.ticket.mark_terminal(terminal);
            release_fence_pins(&entry);

            let mut state = self.lock();
            state.in_flight = state.in_flight.saturating_sub(1);
            let observer = state
                .active
                .take()
                .and_then(|active| active.terminal_observer)
                .or_else(|| entry.terminal_observer.clone());
            release_reservation(&mut state, entry.retained_bytes);
            state.worker_wall = state.worker_wall.saturating_add(elapsed);
            match terminal {
                BoundedKeyedIoTerminal::Succeeded => {
                    state.completed = state.completed.saturating_add(1)
                }
                _ => state.failed = state.failed.saturating_add(1),
            }
            drop(state);
            self.changed.notify_all();
            notify_observer(observer, terminal);
        }
    }

    fn next_entry(&self) -> Option<WorkEntry> {
        let mut state = self.lock();
        if !front_is_runnable(&state) {
            state.pump_active = false;
            state.active_handles.retain(|handle| !handle.is_complete());
            drop(state);
            self.changed.notify_all();
            return None;
        }
        let entry = state.queue.pop_front();
        if let Some(entry) = &entry {
            if entry.ticket.terminal().is_none() {
                if entry.deadline.expired(Instant::now()) {
                    entry
                        .ticket
                        .mark_terminal_before_start(BoundedKeyedIoTerminal::DeadlineBeforeStart);
                } else if entry.fence {
                    if let Some(failure) = fence_prerequisite_failure(&entry.prerequisites) {
                        entry
                            .ticket
                            .mark_terminal(BoundedKeyedIoTerminal::Failed(failure));
                    } else {
                        entry.ticket.mark_started();
                    }
                } else {
                    entry.ticket.mark_started();
                }
            }
            state.in_flight = state.in_flight.saturating_add(1);
            state.active = Some(ActiveEntry {
                key: entry.key.clone(),
                generation: entry.generation,
                epoch: entry.epoch,
                enqueued_at: entry.enqueued_at,
                ticket: entry.ticket.clone(),
                terminal_observer: entry.terminal_observer.clone(),
                fence: entry.fence,
            });
        }
        entry
    }

    fn finish_skipped(&self, entry: WorkEntry) {
        release_fence_pins(&entry);
        let terminal = entry
            .ticket
            .terminal()
            .unwrap_or(BoundedKeyedIoTerminal::Shutdown);
        let mut state = self.lock();
        state.in_flight = state.in_flight.saturating_sub(1);
        let observer = state
            .active
            .take()
            .and_then(|active| active.terminal_observer)
            .or_else(|| entry.terminal_observer.clone());
        release_reservation(&mut state, entry.retained_bytes);
        match terminal {
            BoundedKeyedIoTerminal::Failed(_) => state.failed = state.failed.saturating_add(1),
            _ => state.cancelled = state.cancelled.saturating_add(1),
        }
        drop(state);
        self.changed.notify_all();
        notify_observer(observer, terminal);
    }

    fn lock(&self) -> MutexGuard<'_, LaneState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn reserve(
    lane: &LaneInner,
    state: &mut LaneState,
    retained_bytes: usize,
) -> Result<(), BoundedKeyedIoAdmissionError> {
    if !state.accepting {
        return Err(BoundedKeyedIoAdmissionError::Closed);
    }
    if state.reserved_entries >= lane.limits.max_entries {
        return Err(BoundedKeyedIoAdmissionError::EntryCapacityExceeded);
    }
    let next_bytes = state
        .retained_bytes
        .checked_add(retained_bytes)
        .ok_or(BoundedKeyedIoAdmissionError::RetainedBytesOverflow)?;
    if next_bytes > lane.limits.max_retained_bytes {
        return Err(BoundedKeyedIoAdmissionError::RetainedBytesCapacityExceeded);
    }
    state.reserved_entries += 1;
    state.retained_bytes = next_bytes;
    state.submitted = state.submitted.saturating_add(1);
    Ok(())
}

pub(super) fn release_reservation(state: &mut LaneState, retained_bytes: usize) {
    state.reserved_entries = state.reserved_entries.saturating_sub(1);
    state.retained_bytes = state.retained_bytes.saturating_sub(retained_bytes);
}

fn take_ticket_id(state: &mut LaneState) -> u64 {
    let id = state.next_ticket_id;
    state.next_ticket_id = state.next_ticket_id.saturating_add(1);
    id
}

fn mark_pump_needed(state: &mut LaneState) -> bool {
    if state.pump_active || !front_is_runnable(state) {
        false
    } else {
        state.pump_active = true;
        true
    }
}

fn front_is_runnable(state: &LaneState) -> bool {
    let Some(front) = state.queue.front() else {
        return false;
    };
    !state.suspended.iter().any(|(ticket_id, suspended)| {
        suspended.epoch < front.epoch
            || (suspended.epoch == front.epoch && (front.fence || *ticket_id < front.ticket.id()))
    })
}

fn finish_pre_start_entry(
    state: &mut LaneState,
    entry: WorkEntry,
    requested_terminal: BoundedKeyedIoTerminal,
    notifications: &mut Vec<TerminalNotification>,
) {
    let terminal = if entry.ticket.mark_terminal_before_start(requested_terminal) {
        requested_terminal
    } else {
        entry.ticket.terminal().unwrap_or(requested_terminal)
    };
    release_reservation(state, entry.retained_bytes);
    state.cancelled = state.cancelled.saturating_add(1);
    notifications.push(TerminalNotification {
        observer: entry.terminal_observer,
        terminal,
    });
}

fn notify_observers(notifications: Vec<TerminalNotification>) {
    for notification in notifications {
        notify_observer(notification.observer, notification.terminal);
    }
}

fn notify_observer(observer: Option<TerminalObserver>, terminal: BoundedKeyedIoTerminal) {
    if let Some(observer) = observer {
        let _ = catch_unwind(AssertUnwindSafe(|| observer(terminal)));
    }
}
