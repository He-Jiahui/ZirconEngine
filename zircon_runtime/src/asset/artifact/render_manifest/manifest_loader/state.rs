use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Instant;

use crate::core::runtime::TaskHandle;

use super::super::io_frontier::{RenderArtifactIoDemandKey, RenderArtifactIoFrontier};
use super::super::{RenderArtifactIoPriority, RenderArtifactManifest};
use super::contract::{
    RenderArtifactManifestCancelReason, RenderArtifactManifestFailure,
    RenderArtifactManifestLoadStage, RenderArtifactManifestPoll, RenderArtifactManifestRequestKey,
};
use super::loader::{RenderArtifactManifestLoaderInner, RenderArtifactManifestTicket};

pub(super) const TICKET_ACTIVE: u8 = 0;
pub(super) const TICKET_CALLER_CANCELLED: u8 = 1;
pub(super) const TICKET_EXPIRED: u8 = 2;
pub(super) const TICKET_OWNER_CLOSED: u8 = 3;

pub(super) struct RenderArtifactManifestEntry {
    key: RenderArtifactManifestRequestKey,
    retained_bytes: usize,
    state: Mutex<RenderArtifactManifestEntryState>,
}

struct RenderArtifactManifestEntryState {
    outcome: RenderArtifactManifestEntryOutcome,
    ticket_count: usize,
    task: Option<TaskHandle>,
}

#[derive(Clone)]
enum RenderArtifactManifestEntryOutcome {
    Pending(RenderArtifactManifestLoadStage),
    Ready(Arc<RenderArtifactManifest>),
    Failed(Arc<RenderArtifactManifestFailure>),
    Cancelled(RenderArtifactManifestCancelReason),
}

impl RenderArtifactManifestEntry {
    pub(super) fn new(
        key: RenderArtifactManifestRequestKey,
        retained_bytes: usize,
        ticket_count: usize,
    ) -> Self {
        Self {
            key,
            retained_bytes,
            state: Mutex::new(RenderArtifactManifestEntryState {
                outcome: RenderArtifactManifestEntryOutcome::Pending(
                    RenderArtifactManifestLoadStage::QueuedIo,
                ),
                ticket_count,
                task: None,
            }),
        }
    }

    pub(super) const fn key(&self) -> &RenderArtifactManifestRequestKey {
        &self.key
    }

    pub(super) const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(super) fn poll(&self) -> RenderArtifactManifestPoll {
        match self.lock().outcome.clone() {
            RenderArtifactManifestEntryOutcome::Pending(stage) => {
                RenderArtifactManifestPoll::Pending(stage)
            }
            RenderArtifactManifestEntryOutcome::Ready(manifest) => {
                RenderArtifactManifestPoll::Ready(manifest)
            }
            RenderArtifactManifestEntryOutcome::Failed(failure) => {
                RenderArtifactManifestPoll::Failed(failure)
            }
            RenderArtifactManifestEntryOutcome::Cancelled(reason) => {
                RenderArtifactManifestPoll::Cancelled(reason)
            }
        }
    }

    pub(super) fn ticket_count(&self) -> usize {
        self.lock().ticket_count
    }

    pub(super) fn add_tickets(&self, count: usize) {
        let mut state = self.lock();
        state.ticket_count = state.ticket_count.saturating_add(count);
    }

    pub(super) fn remove_ticket(&self) -> bool {
        let mut state = self.lock();
        state.ticket_count = state.ticket_count.saturating_sub(1);
        state.ticket_count == 0
    }

    pub(super) fn install_task(&self, task: TaskHandle) {
        let mut state = self.lock();
        if !matches!(
            &state.outcome,
            RenderArtifactManifestEntryOutcome::Pending(_)
        ) {
            return;
        }
        state.task = Some(task);
    }

    pub(super) fn begin_io(&self) -> bool {
        let mut state = self.lock();
        if !matches!(
            &state.outcome,
            RenderArtifactManifestEntryOutcome::Pending(RenderArtifactManifestLoadStage::QueuedIo)
        ) {
            return false;
        }
        state.outcome =
            RenderArtifactManifestEntryOutcome::Pending(RenderArtifactManifestLoadStage::Reading);
        true
    }

    pub(super) fn complete(&self, manifest: Arc<RenderArtifactManifest>) -> bool {
        let mut state = self.lock();
        if !matches!(
            &state.outcome,
            RenderArtifactManifestEntryOutcome::Pending(RenderArtifactManifestLoadStage::Reading)
        ) {
            return false;
        }
        state.outcome = RenderArtifactManifestEntryOutcome::Ready(manifest);
        true
    }

    pub(super) fn fail(&self, failure: RenderArtifactManifestFailure) -> bool {
        let mut state = self.lock();
        if !matches!(
            &state.outcome,
            RenderArtifactManifestEntryOutcome::Pending(_)
        ) {
            return false;
        }
        state.outcome = RenderArtifactManifestEntryOutcome::Failed(Arc::new(failure));
        true
    }

    pub(super) fn cancel(&self, reason: RenderArtifactManifestCancelReason) -> bool {
        let task = {
            let mut state = self.lock();
            if !matches!(
                &state.outcome,
                RenderArtifactManifestEntryOutcome::Pending(_)
            ) {
                return false;
            }
            state.outcome = RenderArtifactManifestEntryOutcome::Cancelled(reason);
            state.task.take()
        };
        drop(task);
        true
    }

    fn lock(&self) -> MutexGuard<'_, RenderArtifactManifestEntryState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(super) struct TicketRegistration {
    status: AtomicU8,
}

impl TicketRegistration {
    fn active() -> Self {
        Self {
            status: AtomicU8::new(TICKET_ACTIVE),
        }
    }

    pub(super) fn status(&self) -> u8 {
        self.status.load(Ordering::Acquire)
    }

    pub(super) fn transition_from_active(&self, status: u8) -> bool {
        self.status
            .compare_exchange(TICKET_ACTIVE, status, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

pub(super) struct RegistryTicket {
    pub(super) entry: Weak<RenderArtifactManifestEntry>,
    pub(super) key: RenderArtifactManifestRequestKey,
    pub(super) demand: RenderArtifactIoDemandKey,
    pub(super) deadline: Option<Instant>,
    pub(super) registration: Arc<TicketRegistration>,
}

pub(super) struct RenderArtifactManifestRegistry {
    pub(super) accepting: bool,
    pub(super) next_ticket_id: u64,
    pub(super) next_task_id: u64,
    pub(super) next_frontier_sequence: u64,
    pub(super) reserved_retained_bytes: usize,
    pub(super) entries: HashMap<RenderArtifactManifestRequestKey, Arc<RenderArtifactManifestEntry>>,
    pub(super) tickets: HashMap<u64, RegistryTicket>,
    pub(super) deadlines: BTreeSet<(Instant, u64)>,
    pub(super) io_frontier: RenderArtifactIoFrontier<RenderArtifactManifestRequestKey>,
}

impl RenderArtifactManifestRegistry {
    pub(super) fn new() -> Self {
        Self {
            accepting: true,
            next_ticket_id: 1,
            next_task_id: 1,
            next_frontier_sequence: 1,
            reserved_retained_bytes: 0,
            entries: HashMap::new(),
            tickets: HashMap::new(),
            deadlines: BTreeSet::new(),
            io_frontier: RenderArtifactIoFrontier::new(),
        }
    }
}

pub(super) fn register_ticket(
    loader: &Arc<RenderArtifactManifestLoaderInner>,
    registry: &mut RenderArtifactManifestRegistry,
    ticket_id: u64,
    key: RenderArtifactManifestRequestKey,
    entry: &Arc<RenderArtifactManifestEntry>,
    priority: RenderArtifactIoPriority,
    deadline: Option<Instant>,
) -> RenderArtifactManifestTicket {
    let registration = Arc::new(TicketRegistration::active());
    let demand = RenderArtifactIoDemandKey::new(priority, deadline, ticket_id);
    registry.tickets.insert(
        ticket_id,
        RegistryTicket {
            entry: Arc::downgrade(entry),
            key: key.clone(),
            demand,
            deadline,
            registration: Arc::clone(&registration),
        },
    );
    registry.io_frontier.add_waiter(key, demand);
    if let Some(deadline) = deadline {
        registry.deadlines.insert((deadline, ticket_id));
    }
    RenderArtifactManifestTicket::from_parts(
        ticket_id,
        Arc::clone(entry),
        registration,
        Arc::downgrade(loader),
    )
}

pub(super) fn remove_registered_ticket(
    registry: &mut RenderArtifactManifestRegistry,
    ticket_id: u64,
) -> Option<RegistryTicket> {
    let ticket = registry.tickets.remove(&ticket_id)?;
    if let Some(deadline) = ticket.deadline {
        registry.deadlines.remove(&(deadline, ticket_id));
    }
    registry
        .io_frontier
        .remove_waiter(&ticket.key, ticket.demand);
    Some(ticket)
}

pub(super) fn remove_entry(
    registry: &mut RenderArtifactManifestRegistry,
    entry: &Arc<RenderArtifactManifestEntry>,
) {
    if registry
        .entries
        .get(entry.key())
        .is_some_and(|current| Arc::ptr_eq(current, entry))
    {
        registry.entries.remove(entry.key());
        registry.io_frontier.remove_entry(entry.key());
        registry.reserved_retained_bytes = registry
            .reserved_retained_bytes
            .saturating_sub(entry.retained_bytes());
    }
}

pub(super) fn take_task_id(
    registry: &mut RenderArtifactManifestRegistry,
) -> Result<u64, super::contract::RenderArtifactManifestIoDispatchError> {
    let task_id = registry.next_task_id;
    registry.next_task_id = registry
        .next_task_id
        .checked_add(1)
        .ok_or(super::contract::RenderArtifactManifestIoDispatchError::TaskIdExhausted)?;
    Ok(task_id)
}
