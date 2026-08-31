use std::collections::{BTreeSet, HashMap, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use super::{
    BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority,
    BoundedKeyedIoDiagnostics, BoundedKeyedIoFailure, BoundedKeyedIoFence, BoundedKeyedIoKey,
    BoundedKeyedIoTerminal, BoundedKeyedIoTicket, BoundedKeyedIoWork, BoundedKeyedIoWorkDeadline,
    GlobalAdmissionEpoch,
};
#[cfg(test)]
use crate::core::runtime::tasks::TaskPool;
use crate::core::runtime::tasks::{JobScheduler, TaskTimer};

mod coalescing;
mod fence_prerequisites;
mod queue;
mod shutdown;
mod state;

use coalescing::{coalesce_queued_generation, insert_ordered};
use fence_prerequisites::{
    capture_fence_prerequisites, fence_prerequisite_failure, plan_fence_prerequisites,
    release_fence_pins,
};
#[cfg(test)]
pub(super) use queue::merge_ordered;
use queue::{
    finish_pre_start_entry, front_is_runnable, mark_pump_needed, merge_ordered_queue,
    notify_observer, notify_observers, release_reservation, remove_suspended_entry, reserve,
    take_ticket_id,
};
use shutdown::diagnostics_for_state;
pub use shutdown::BoundedKeyedIoShutdownGuard;
pub(super) use state::LaneInner;
use state::{
    ActiveEntry, FencePrerequisite, LaneState, TerminalNotification, TerminalObserver, WorkEntry,
};

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
                    suspended_order: BTreeSet::new(),
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
        key: impl Into<BoundedKeyedIoKey>,
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
            let previous = state.suspended.insert(
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
            debug_assert!(previous.is_none(), "ticket id must be unique");
            let inserted = state.suspended_order.insert((epoch, id));
            debug_assert!(inserted, "suspended order key must be unique");
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

    #[cfg(test)]
    pub(crate) fn shares_execution_owner_with(&self, pool: &TaskPool) -> bool {
        self.inner.scheduler.shares_execution_owner_with(pool)
    }

    pub fn shutdown(&self) -> BoundedKeyedIoShutdownGuard {
        let (notifications, start_pump) = {
            let mut state = self.inner.lock();
            state.accepting = false;
            let mut notifications = Vec::new();

            let mut suspended = std::mem::take(&mut state.suspended);
            let suspended_order = std::mem::take(&mut state.suspended_order);
            debug_assert_eq!(suspended_order.len(), suspended.len());
            let mut retained_suspended = VecDeque::new();
            for (_, ticket_id) in suspended_order {
                let entry = suspended
                    .remove(&ticket_id)
                    .expect("suspended order index must mirror ticket storage");
                if entry.fence || entry.ticket.fence_pinned() {
                    retained_suspended.push_back(entry);
                } else {
                    finish_pre_start_entry(
                        &mut state,
                        entry,
                        BoundedKeyedIoTerminal::Shutdown,
                        &mut notifications,
                    );
                }
            }
            assert!(
                suspended.is_empty(),
                "suspended order index must cover ticket storage"
            );
            merge_ordered_queue(&mut state.queue, retained_suspended);

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
            let Some(entry) = remove_suspended_entry(&mut state, ticket_id) else {
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
            let Some(entry) = remove_suspended_entry(&mut state, ticket_id) else {
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
        if let Some(entry) = remove_suspended_entry(&mut state, ticket_id) {
            release_reservation(&mut state, entry.retained_bytes);
            state.submitted = state.submitted.saturating_sub(1);
        }
        drop(state);
        lane.changed.notify_all();
    }

    fn expire_before_start(lane: &Arc<Self>, ticket_id: u64) {
        let (notification, start_pump) = {
            let mut state = lane.lock();
            let entry = remove_suspended_entry(&mut state, ticket_id).or_else(|| {
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
}

#[cfg(test)]
impl BoundedKeyedIoLane {
    pub(super) fn front_is_runnable_for_tests(&self) -> bool {
        front_is_runnable(&self.inner.lock())
    }

    pub(super) fn suspended_order_index_matches_for_tests(&self) -> bool {
        let state = self.inner.lock();
        state.suspended.len() == state.suspended_order.len()
            && state.suspended.iter().all(|(ticket_id, entry)| {
                state.suspended_order.contains(&(entry.epoch, *ticket_id))
            })
    }

    pub(super) fn front_readiness_snapshot_for_tests(
        &self,
    ) -> Option<(
        GlobalAdmissionEpoch,
        u64,
        bool,
        Vec<(GlobalAdmissionEpoch, u64)>,
    )> {
        let state = self.inner.lock();
        let front = state.queue.front()?;
        Some((
            front.epoch,
            front.ticket.id(),
            front.fence,
            state
                .suspended
                .iter()
                .map(|(ticket_id, entry)| (entry.epoch, *ticket_id))
                .collect(),
        ))
    }
}
