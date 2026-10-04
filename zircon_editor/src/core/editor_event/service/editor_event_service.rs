use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::editor_event::{
    EditorEventJournal, EditorEventJournalStore, EditorEventListenerRegistry, EditorEventRecord,
    EditorEventRetentionPolicy, SharedEditorEventRecord,
};
use crate::core::editor_message::SharedEditorMessageBus;

use super::state::EditorEventSequenceState;
use super::EditorEventStamp;

/// Journal, listener, sequence, and revision owner for editor events.
pub struct EditorEventService {
    sequence_state: Mutex<EditorEventSequenceState>,
    journal: Mutex<EditorEventJournalStore>,
    listeners: Mutex<EditorEventListenerRegistry>,
    bus: SharedEditorMessageBus,
}

impl EditorEventService {
    pub fn new(bus: SharedEditorMessageBus) -> Self {
        Self::with_retention_policy(bus, EditorEventRetentionPolicy::default())
    }

    pub fn with_retention_policy(
        bus: SharedEditorMessageBus,
        retention_policy: EditorEventRetentionPolicy,
    ) -> Self {
        Self {
            sequence_state: Mutex::new(EditorEventSequenceState::default()),
            journal: Mutex::new(EditorEventJournalStore::new(retention_policy.journal)),
            listeners: Mutex::new(EditorEventListenerRegistry::new(retention_policy.listeners)),
            bus,
        }
    }

    pub fn bus(&self) -> &SharedEditorMessageBus {
        &self.bus
    }

    pub(crate) fn begin_event(&self) -> EditorEventStamp {
        self.allocate_stamp(true)
    }

    pub(crate) fn begin_observation(&self) -> EditorEventStamp {
        self.allocate_stamp(false)
    }

    pub(crate) fn record(&self, record: EditorEventRecord) {
        let record = Arc::new(SharedEditorEventRecord::new(record));
        {
            self.lock_journal().push(Arc::clone(&record));
        }
        let routes = { self.lock_listeners().delivery_routes() };
        deliver_matching_arc(
            &routes,
            record,
            |route, record| route.accepts(record.record()),
            |route, record| route.enqueue(record),
        );
    }

    pub fn journal(&self) -> EditorEventJournal {
        self.lock_journal().snapshot()
    }

    fn allocate_stamp(&self, advances_revision: bool) -> EditorEventStamp {
        let mut state = self.lock_sequence_state();
        state.next_event_id = state.next_event_id.saturating_add(1);
        state.next_sequence = state.next_sequence.saturating_add(1);
        let before_revision = state.revision;
        if advances_revision {
            state.revision = state.revision.saturating_add(1);
        }
        EditorEventStamp {
            event_id: crate::core::editor_event::EditorEventId::new(state.next_event_id),
            sequence: crate::core::editor_event::EditorEventSequence::new(state.next_sequence),
            before_revision,
            after_revision: state.revision,
        }
    }

    fn lock_sequence_state(&self) -> MutexGuard<'_, EditorEventSequenceState> {
        self.sequence_state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_journal(&self) -> MutexGuard<'_, EditorEventJournalStore> {
        self.journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_listeners(&self) -> MutexGuard<'_, EditorEventListenerRegistry> {
        self.listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[inline]
fn deliver_matching_arc<T, R>(
    routes: &[R],
    payload: Arc<T>,
    mut accepts: impl FnMut(&R, &T) -> bool,
    mut deliver: impl FnMut(&R, Arc<T>),
) {
    let mut pending_route = None;
    for route in routes {
        if accepts(route, &payload) {
            if let Some(previous_route) = pending_route.replace(route) {
                deliver(previous_route, Arc::clone(&payload));
            }
        }
    }
    if let Some(route) = pending_route {
        deliver(route, payload);
    }
}

#[cfg(test)]
#[path = "tests/editor_event_service.rs"]
mod tests;
