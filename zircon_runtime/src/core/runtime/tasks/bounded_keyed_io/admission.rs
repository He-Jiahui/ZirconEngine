use std::sync::Arc;
use std::time::Instant;

use super::lane::{LaneInner, WorkEntry};
use super::{BoundedKeyedIoFailure, BoundedKeyedIoTicket, GlobalAdmissionEpoch};

pub type BoundedKeyedIoWork =
    Box<dyn FnOnce() -> Result<(), BoundedKeyedIoFailure> + Send + 'static>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoundedKeyedIoWorkDeadline {
    deadline: Option<Instant>,
}

impl BoundedKeyedIoWorkDeadline {
    pub const fn none() -> Self {
        Self { deadline: None }
    }

    pub const fn at(deadline: Instant) -> Self {
        Self {
            deadline: Some(deadline),
        }
    }

    pub(crate) fn expired(self, now: Instant) -> bool {
        self.deadline.is_some_and(|deadline| now >= deadline)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundedKeyedIoAdmissionError {
    Closed,
    EntryCapacityExceeded,
    RetainedBytesCapacityExceeded,
    RetainedBytesOverflow,
}

#[derive(Clone, Debug)]
pub struct BoundedKeyedIoCancelAuthority {
    ticket_id: u64,
    _private: (),
}

impl BoundedKeyedIoCancelAuthority {
    pub(crate) const fn new(ticket_id: u64) -> Self {
        Self {
            ticket_id,
            _private: (),
        }
    }

    pub(crate) const fn ticket_id(&self) -> u64 {
        self.ticket_id
    }
}

pub struct BoundedKeyedIoAdmission {
    pub(crate) lane: Arc<LaneInner>,
    pub(crate) entry: Option<WorkEntry>,
    pub(crate) ticket: BoundedKeyedIoTicket,
    pub(crate) cancel_authority: BoundedKeyedIoCancelAuthority,
}

impl BoundedKeyedIoAdmission {
    pub fn ticket(&self) -> BoundedKeyedIoTicket {
        self.ticket.clone()
    }

    pub fn cancel_authority(&self) -> BoundedKeyedIoCancelAuthority {
        self.cancel_authority.clone()
    }

    pub fn epoch(&self) -> GlobalAdmissionEpoch {
        self.entry
            .as_ref()
            .map_or(GlobalAdmissionEpoch::initial(), |entry| entry.epoch)
    }

    pub fn activate(mut self) -> BoundedKeyedIoTicket {
        if let Some(entry) = self.entry.take() {
            LaneInner::activate(&self.lane, entry);
        }
        self.ticket.clone()
    }
}

impl Drop for BoundedKeyedIoAdmission {
    fn drop(&mut self) {
        if let Some(entry) = self.entry.take() {
            LaneInner::release_unactivated(&self.lane, entry);
        }
    }
}
