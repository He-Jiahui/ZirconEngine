use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::EventTypeId;

pub(crate) struct EventReaderLeaseRegistry {
    reader_count: AtomicU32,
    next_generation: AtomicU64,
}

impl EventReaderLeaseRegistry {
    pub(crate) const fn new() -> Self {
        Self {
            reader_count: AtomicU32::new(0),
            next_generation: AtomicU64::new(1),
        }
    }

    pub(crate) fn acquire(
        self: &Arc<Self>,
        event_type_id: EventTypeId,
    ) -> Option<EventReaderLease> {
        let generation = self
            .next_generation
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |generation| {
                generation.checked_add(1)
            })
            .ok()?;
        if self
            .reader_count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
            .is_err()
        {
            return None;
        }
        Some(EventReaderLease {
            event_type_id,
            generation,
            registry: Arc::clone(self),
            connected: true,
        })
    }

    pub(crate) fn reader_count(&self) -> u32 {
        self.reader_count.load(Ordering::Acquire)
    }

    pub(crate) fn connect_untracked(&self) -> bool {
        self.reader_count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
            .is_ok()
    }

    pub(crate) fn disconnect_untracked(&self) -> bool {
        self.reader_count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_sub(1)
            })
            .is_ok()
    }
}

/// An owner-bound registration in a single event channel.
///
/// A lease is deliberately non-cloneable. Its destructor is the final safety
/// net for direct `SystemState` ownership; schedule-owned systems should still
/// retire explicitly while their `World` is available.
pub struct EventReaderLease {
    event_type_id: EventTypeId,
    generation: u64,
    registry: Arc<EventReaderLeaseRegistry>,
    connected: bool,
}

impl EventReaderLease {
    pub fn event_type_id(&self) -> EventTypeId {
        self.event_type_id
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn belongs_to(&self, registry: &Arc<EventReaderLeaseRegistry>) -> bool {
        Arc::ptr_eq(&self.registry, registry)
    }

    pub(crate) fn disconnect(&mut self) -> bool {
        if !self.connected {
            return false;
        }
        if !self.registry.disconnect_untracked() {
            return false;
        }
        self.connected = false;
        true
    }
}

impl Drop for EventReaderLease {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

impl std::fmt::Debug for EventReaderLease {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EventReaderLease")
            .field("event_type_id", &self.event_type_id)
            .field("generation", &self.generation)
            .field("connected", &self.connected)
            .finish_non_exhaustive()
    }
}
