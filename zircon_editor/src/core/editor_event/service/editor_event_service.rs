use std::sync::{Mutex, MutexGuard};

use crate::core::editor_event::{EditorEventJournal, EditorEventRecord};
use crate::core::editor_message::SharedEditorMessageBus;

use super::state::EditorEventServiceState;
use super::EditorEventStamp;

/// Journal, listener, sequence, and revision owner for editor events.
pub struct EditorEventService {
    state: Mutex<EditorEventServiceState>,
    bus: SharedEditorMessageBus,
}

impl EditorEventService {
    pub fn new(bus: SharedEditorMessageBus) -> Self {
        Self {
            state: Mutex::new(EditorEventServiceState::default()),
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
        let mut state = self.lock_state();
        state.journal.push(record.clone());
        state.listeners.notify(&record);
    }

    pub fn journal(&self) -> EditorEventJournal {
        self.lock_state().journal.clone()
    }

    fn allocate_stamp(&self, advances_revision: bool) -> EditorEventStamp {
        let mut state = self.lock_state();
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

    pub(super) fn lock_state(&self) -> MutexGuard<'_, EditorEventServiceState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
