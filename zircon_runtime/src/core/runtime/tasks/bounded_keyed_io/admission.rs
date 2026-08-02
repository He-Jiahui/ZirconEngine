use std::sync::Arc;
use std::time::Instant;

use super::lane::LaneInner;
use super::{
    BoundedKeyedIoFailure, BoundedKeyedIoTerminal, BoundedKeyedIoTicket, GlobalAdmissionEpoch,
};

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

    pub(crate) const fn instant(self) -> Option<Instant> {
        self.deadline
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundedKeyedIoAdmissionError {
    Closed,
    EntryCapacityExceeded,
    RetainedBytesCapacityExceeded,
    RetainedBytesOverflow,
    DeadlineTimerUnavailable,
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
    pub(crate) ticket_id: u64,
    pub(crate) epoch: GlobalAdmissionEpoch,
    pub(crate) armed: bool,
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
        self.epoch
    }

    pub fn observe_terminal(
        &self,
        observer: impl Fn(BoundedKeyedIoTerminal) + Send + Sync + 'static,
    ) {
        LaneInner::observe_terminal(&self.lane, self.ticket_id, &self.ticket, Arc::new(observer));
    }

    pub fn activate(mut self) -> BoundedKeyedIoTicket {
        if self.armed {
            self.armed = false;
            LaneInner::activate(&self.lane, self.ticket_id);
        }
        self.ticket.clone()
    }
}

impl Drop for BoundedKeyedIoAdmission {
    fn drop(&mut self) {
        if self.armed {
            self.armed = false;
            LaneInner::release_unactivated(&self.lane, self.ticket_id);
        }
    }
}
