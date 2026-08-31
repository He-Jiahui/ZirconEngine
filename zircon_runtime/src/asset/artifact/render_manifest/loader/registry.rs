use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Weak};
use std::time::Instant;

use super::super::io_frontier::{RenderArtifactIoDemandKey, RenderArtifactIoFrontier};
use super::super::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactContentId,
};
use super::entry::RenderArtifactBlockEntry;
use super::loader::{RenderArtifactBlockLoaderInner, RenderArtifactBlockTicket};

pub(super) const TICKET_ACTIVE: u8 = 0;
pub(super) const TICKET_CALLER_CANCELLED: u8 = 1;
pub(super) const TICKET_EXPIRED: u8 = 2;
pub(super) const TICKET_OWNER_CLOSED: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct RenderArtifactDecodeKey {
    content_id: RenderArtifactContentId,
    codec: RenderArtifactBlockCodec,
    encoded_bytes: u64,
    decoded_bytes: u64,
}

impl RenderArtifactDecodeKey {
    pub(super) fn from_descriptor(descriptor: &RenderArtifactBlockDescriptor) -> Self {
        Self {
            content_id: descriptor.content_id(),
            codec: descriptor.codec(),
            encoded_bytes: descriptor.encoded_bytes(),
            decoded_bytes: descriptor.decoded_bytes(),
        }
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
    pub(super) entry: Weak<RenderArtifactBlockEntry>,
    pub(super) key: RenderArtifactDecodeKey,
    pub(super) demand: RenderArtifactIoDemandKey,
    pub(super) deadline: Option<Instant>,
    pub(super) registration: Arc<TicketRegistration>,
}

pub(super) struct RenderArtifactBlockRegistry {
    pub(super) accepting: bool,
    pub(super) next_ticket_id: u64,
    pub(super) next_task_id: u64,
    pub(super) next_frontier_sequence: u64,
    pub(super) retained_bytes: usize,
    pub(super) entries: HashMap<RenderArtifactDecodeKey, Arc<RenderArtifactBlockEntry>>,
    pub(super) tickets: HashMap<u64, RegistryTicket>,
    pub(super) deadlines: BTreeSet<(Instant, u64)>,
    pub(super) io_frontier: RenderArtifactIoFrontier<RenderArtifactDecodeKey>,
}

impl RenderArtifactBlockRegistry {
    pub(super) fn new() -> Self {
        Self {
            accepting: true,
            next_ticket_id: 1,
            next_task_id: 1,
            next_frontier_sequence: 1,
            retained_bytes: 0,
            entries: HashMap::new(),
            tickets: HashMap::new(),
            deadlines: BTreeSet::new(),
            io_frontier: RenderArtifactIoFrontier::new(),
        }
    }
}

pub(super) fn register_ticket(
    loader: &Arc<RenderArtifactBlockLoaderInner>,
    registry: &mut RenderArtifactBlockRegistry,
    ticket_id: u64,
    key: RenderArtifactDecodeKey,
    entry: &Arc<RenderArtifactBlockEntry>,
    descriptor: RenderArtifactBlockDescriptor,
    priority: super::super::RenderArtifactIoPriority,
    deadline: Option<Instant>,
) -> RenderArtifactBlockTicket {
    let registration = Arc::new(TicketRegistration::active());
    let demand = RenderArtifactIoDemandKey::new(priority, deadline, ticket_id);
    registry.tickets.insert(
        ticket_id,
        RegistryTicket {
            entry: Arc::downgrade(entry),
            key,
            demand,
            deadline,
            registration: Arc::clone(&registration),
        },
    );
    registry.io_frontier.add_waiter(key, demand);
    if let Some(deadline) = deadline {
        registry.deadlines.insert((deadline, ticket_id));
    }
    RenderArtifactBlockTicket::from_parts(
        ticket_id,
        Arc::clone(entry),
        descriptor,
        registration,
        Arc::downgrade(loader),
    )
}

pub(super) fn remove_registered_ticket(
    registry: &mut RenderArtifactBlockRegistry,
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
    registry: &mut RenderArtifactBlockRegistry,
    entry: &Arc<RenderArtifactBlockEntry>,
) {
    let key = RenderArtifactDecodeKey::from_descriptor(entry.descriptor());
    if registry
        .entries
        .get(&key)
        .is_some_and(|current| Arc::ptr_eq(current, entry))
    {
        registry.entries.remove(&key);
        registry.io_frontier.remove_entry(&key);
        registry.retained_bytes = registry
            .retained_bytes
            .saturating_sub(entry.retained_bytes());
    }
}

pub(super) fn take_task_id(registry: &mut RenderArtifactBlockRegistry) -> Option<u64> {
    let task_id = registry.next_task_id;
    registry.next_task_id = registry.next_task_id.checked_add(1)?;
    Some(task_id)
}
