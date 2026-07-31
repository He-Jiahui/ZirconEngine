use std::collections::{BTreeSet, HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::{
    BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority,
    BoundedKeyedIoDiagnostics, BoundedKeyedIoFailure, BoundedKeyedIoFence, BoundedKeyedIoTerminal,
    BoundedKeyedIoTicket, BoundedKeyedIoWork, BoundedKeyedIoWorkDeadline, GlobalAdmissionEpoch,
};
use crate::core::runtime::tasks::{JobHandle, JobScheduler};

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
}

struct LaneState {
    accepting: bool,
    pump_active: bool,
    next_ticket_id: u64,
    current_epoch: GlobalAdmissionEpoch,
    reserved_entries: usize,
    retained_bytes: usize,
    in_flight: usize,
    suspended: HashMap<u64, SuspendedEntry>,
    active: Option<ActiveEntry>,
    failed_epochs: BTreeSet<GlobalAdmissionEpoch>,
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

pub(crate) struct WorkEntry {
    pub key: Option<Arc<str>>,
    pub generation: u64,
    pub epoch: GlobalAdmissionEpoch,
    pub retained_bytes: usize,
    pub enqueued_at: Instant,
    pub deadline: BoundedKeyedIoWorkDeadline,
    pub ticket: BoundedKeyedIoTicket,
    pub work: Option<BoundedKeyedIoWork>,
    pub fence: bool,
}

struct SuspendedEntry {
    epoch: GlobalAdmissionEpoch,
    ticket: BoundedKeyedIoTicket,
}

struct ActiveEntry {
    key: Option<Arc<str>>,
    generation: u64,
    epoch: GlobalAdmissionEpoch,
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
                    failed_epochs: BTreeSet::new(),
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
            }),
        }
    }

    pub fn try_admit(
        &self,
        key: impl Into<Arc<str>>,
        generation: u64,
        retained_bytes: usize,
        deadline: BoundedKeyedIoWorkDeadline,
        work: BoundedKeyedIoWork,
    ) -> Result<BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError> {
        let mut state = self.inner.lock();
        reserve(&self.inner, &mut state, retained_bytes)?;
        let id = take_ticket_id(&mut state);
        let ticket = BoundedKeyedIoTicket::pending(id, generation, false);
        let epoch = state.current_epoch;
        state.suspended.insert(
            id,
            SuspendedEntry {
                epoch,
                ticket: ticket.clone(),
            },
        );
        let entry = WorkEntry {
            key: Some(key.into()),
            generation,
            epoch,
            retained_bytes,
            enqueued_at: Instant::now(),
            deadline,
            ticket: ticket.clone(),
            work: Some(work),
            fence: false,
        };
        Ok(BoundedKeyedIoAdmission {
            lane: Arc::clone(&self.inner),
            entry: Some(entry),
            ticket,
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
            reserve(&self.inner, &mut state, retained_bytes)?;
            let id = take_ticket_id(&mut state);
            let epoch = state.current_epoch;
            state.current_epoch = GlobalAdmissionEpoch(epoch.0.saturating_add(1));
            for suspended in state
                .suspended
                .values()
                .filter(|suspended| suspended.epoch <= epoch)
            {
                suspended.ticket.pin_to_fence();
            }
            for queued in state.queue.iter().filter(|queued| queued.epoch <= epoch) {
                queued.ticket.pin_to_fence();
            }
            let ticket = BoundedKeyedIoTicket::pending(id, 0, true);
            state.queue.push_back(WorkEntry {
                key: None,
                generation: 0,
                epoch,
                retained_bytes,
                enqueued_at: Instant::now(),
                deadline,
                ticket: ticket.clone(),
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
        let oldest_age = state
            .queue
            .front()
            .map_or(Duration::ZERO, |entry| entry.enqueued_at.elapsed());
        BoundedKeyedIoDiagnostics {
            queue_entries: state.reserved_entries,
            retained_bytes: state.retained_bytes,
            in_flight: state.in_flight,
            oldest_age,
            submitted: state.submitted,
            completed: state.completed,
            failed: state.failed,
            cancelled: state.cancelled,
            superseded: state.superseded,
            coalesced: state.coalesced,
            worker_wall: state.worker_wall,
        }
    }

    pub fn shutdown(&self) -> BoundedKeyedIoShutdownGuard {
        let start_pump = {
            let mut state = self.inner.lock();
            state.accepting = false;
            for suspended in state.suspended.values() {
                suspended
                    .ticket
                    .mark_terminal(BoundedKeyedIoTerminal::Shutdown);
            }
            let mut retained = VecDeque::new();
            while let Some(entry) = state.queue.pop_front() {
                if entry.ticket.fence_pinned() {
                    retained.push_back(entry);
                    continue;
                }
                entry.ticket.mark_terminal(BoundedKeyedIoTerminal::Shutdown);
                record_non_durable_epoch(&mut state, &entry);
                release_reservation(&mut state, entry.retained_bytes);
                state.cancelled = state.cancelled.saturating_add(1);
            }
            state.queue = retained;
            mark_pump_needed(&mut state)
        };
        if start_pump {
            LaneInner::start_pump(&self.inner);
        }
        BoundedKeyedIoShutdownGuard {
            lane: Arc::clone(&self.inner),
        }
    }
}

impl LaneInner {
    pub(crate) fn activate(lane: &Arc<Self>, entry: WorkEntry) {
        let start_pump = {
            let mut state = lane.lock();
            state.suspended.remove(&entry.ticket.id());
            if !state.accepting {
                entry.ticket.mark_terminal(BoundedKeyedIoTerminal::Shutdown);
                record_non_durable_epoch(&mut state, &entry);
                release_reservation(&mut state, entry.retained_bytes);
                state.cancelled = state.cancelled.saturating_add(1);
            } else if coalesce_queued_generation(&mut state, &entry) {
                let insertion = state
                    .queue
                    .iter()
                    .position(|queued| {
                        queued.epoch > entry.epoch
                            || (queued.epoch == entry.epoch
                                && (queued.fence || queued.ticket.id() > entry.ticket.id()))
                    })
                    .unwrap_or(state.queue.len());
                state.queue.insert(insertion, entry);
            }
            mark_pump_needed(&mut state)
        };
        if start_pump {
            Self::start_pump(lane);
        }
    }

    pub(crate) fn release_unactivated(lane: &Arc<Self>, entry: WorkEntry) {
        entry
            .ticket
            .mark_terminal(BoundedKeyedIoTerminal::CancelledBeforeStart);
        let start_pump = {
            let mut state = lane.lock();
            state.suspended.remove(&entry.ticket.id());
            if entry.ticket.fence_pinned() {
                record_non_durable_epoch(&mut state, &entry);
            }
            release_reservation(&mut state, entry.retained_bytes);
            state.cancelled = state.cancelled.saturating_add(1);
            mark_pump_needed(&mut state)
        };
        if start_pump {
            Self::start_pump(lane);
        }
    }

    fn start_pump(lane: &Arc<Self>) {
        let lane_for_work = Arc::clone(lane);
        let handle = lane.scheduler.schedule(move || lane_for_work.pump());
        lane.lock().active_handles.push(handle);
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
            if entry.deadline.expired(Instant::now()) {
                entry
                    .ticket
                    .mark_terminal(BoundedKeyedIoTerminal::DeadlineBeforeStart);
                self.finish_skipped(entry);
                continue;
            }
            if entry.fence && self.consume_failed_epochs_through(entry.epoch) {
                entry.ticket.mark_terminal(BoundedKeyedIoTerminal::Failed(
                    BoundedKeyedIoFailure::new("pre_fence_obligation_failed"),
                ));
                self.finish_failed_fence(entry);
                continue;
            }
            if !entry.ticket.mark_started() {
                self.finish_skipped(entry);
                continue;
            }
            let started = Instant::now();
            let result = entry
                .work
                .take()
                .expect("activated bounded I/O entry must own work")();
            let elapsed = started.elapsed();
            let terminal = match result {
                Ok(()) => BoundedKeyedIoTerminal::Succeeded,
                Err(error) => BoundedKeyedIoTerminal::Failed(error),
            };
            entry.ticket.mark_terminal(terminal);
            let mut state = self.lock();
            state.in_flight = state.in_flight.saturating_sub(1);
            state.active = None;
            if !entry.fence && terminal != BoundedKeyedIoTerminal::Succeeded {
                state.failed_epochs.insert(entry.epoch);
            }
            release_reservation(&mut state, entry.retained_bytes);
            state.worker_wall = state.worker_wall.saturating_add(elapsed);
            match terminal {
                BoundedKeyedIoTerminal::Succeeded => {
                    state.completed = state.completed.saturating_add(1)
                }
                _ => state.failed = state.failed.saturating_add(1),
            }
        }
    }

    fn next_entry(&self) -> Option<WorkEntry> {
        let mut state = self.lock();
        if !front_is_runnable(&state) {
            state.pump_active = false;
            state.active_handles.retain(|handle| !handle.is_complete());
            return None;
        }
        let entry = state.queue.pop_front();
        if let Some(entry) = &entry {
            state.in_flight = state.in_flight.saturating_add(1);
            state.active = Some(ActiveEntry {
                key: entry.key.clone(),
                generation: entry.generation,
                epoch: entry.epoch,
            });
        }
        entry
    }

    fn finish_skipped(&self, entry: WorkEntry) {
        let mut state = self.lock();
        state.in_flight = state.in_flight.saturating_sub(1);
        state.active = None;
        record_non_durable_epoch(&mut state, &entry);
        release_reservation(&mut state, entry.retained_bytes);
        state.cancelled = state.cancelled.saturating_add(1);
    }

    fn finish_failed_fence(&self, entry: WorkEntry) {
        let mut state = self.lock();
        state.in_flight = state.in_flight.saturating_sub(1);
        state.active = None;
        release_reservation(&mut state, entry.retained_bytes);
        state.failed = state.failed.saturating_add(1);
    }

    fn consume_failed_epochs_through(&self, epoch: GlobalAdmissionEpoch) -> bool {
        let mut state = self.lock();
        let failed = state.failed_epochs.range(..=epoch).next().is_some();
        if failed {
            state
                .failed_epochs
                .retain(|failed_epoch| *failed_epoch > epoch);
        }
        failed
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

fn release_reservation(state: &mut LaneState, retained_bytes: usize) {
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

fn coalesce_queued_generation(state: &mut LaneState, successor: &WorkEntry) -> bool {
    let Some(key) = successor.key.as_deref() else {
        return true;
    };
    let active_successor = state.active.as_ref().is_some_and(|active| {
        active.epoch == successor.epoch
            && active.key.as_deref() == Some(key)
            && active.generation > successor.generation
    });
    let queued_successor = state.queue.iter().any(|queued| {
        !queued.fence
            && queued.epoch == successor.epoch
            && queued.key.as_deref() == Some(key)
            && queued.generation > successor.generation
    });
    if active_successor || queued_successor {
        let successor_generation = state
            .active
            .iter()
            .filter(|active| active.epoch == successor.epoch && active.key.as_deref() == Some(key))
            .map(|active| active.generation)
            .chain(
                state
                    .queue
                    .iter()
                    .filter(|queued| {
                        !queued.fence
                            && queued.epoch == successor.epoch
                            && queued.key.as_deref() == Some(key)
                    })
                    .map(|queued| queued.generation),
            )
            .max()
            .unwrap_or(successor.generation);
        successor
            .ticket
            .mark_terminal(BoundedKeyedIoTerminal::Superseded {
                successor: successor_generation,
            });
        release_reservation(state, successor.retained_bytes);
        state.superseded = state.superseded.saturating_add(1);
        state.coalesced = state.coalesced.saturating_add(1);
        return false;
    }
    let mut index = 0;
    while index < state.queue.len() {
        let matches = {
            let queued = &state.queue[index];
            !queued.fence && queued.epoch == successor.epoch && queued.key.as_deref() == Some(key)
        };
        if !matches {
            index += 1;
            continue;
        }
        let queued = state
            .queue
            .remove(index)
            .expect("matched queued entry must exist");
        queued
            .ticket
            .mark_terminal(BoundedKeyedIoTerminal::Superseded {
                successor: successor.generation,
            });
        release_reservation(state, queued.retained_bytes);
        state.superseded = state.superseded.saturating_add(1);
        state.coalesced = state.coalesced.saturating_add(1);
    }
    true
}

fn record_non_durable_epoch(state: &mut LaneState, entry: &WorkEntry) {
    if entry.fence {
        return;
    }
    if matches!(
        entry.ticket.terminal(),
        Some(BoundedKeyedIoTerminal::Succeeded | BoundedKeyedIoTerminal::Superseded { .. })
    ) {
        return;
    }
    state.failed_epochs.insert(entry.epoch);
}

pub struct BoundedKeyedIoShutdownGuard {
    lane: Arc<LaneInner>,
}

impl BoundedKeyedIoShutdownGuard {
    pub fn is_complete(&self) -> bool {
        let state = self.lane.lock();
        state.reserved_entries == 0 && state.active_handles.iter().all(JobHandle::is_complete)
    }

    pub fn wait_until(&self, deadline: Instant) -> bool {
        while !self.is_complete() {
            if Instant::now() >= deadline {
                return false;
            }
            std::hint::spin_loop();
        }
        true
    }

    pub fn diagnostics(&self) -> BoundedKeyedIoDiagnostics {
        BoundedKeyedIoLane {
            inner: Arc::clone(&self.lane),
        }
        .diagnostics()
    }
}

impl Drop for BoundedKeyedIoShutdownGuard {
    fn drop(&mut self) {
        while !self.is_complete() {
            let handles = self.lane.lock().active_handles.clone();
            for handle in handles {
                handle.wait();
            }
            std::hint::spin_loop();
        }
    }
}
