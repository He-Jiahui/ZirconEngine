use crate::core::editor_event::{EditorEventJournal, EditorEventListenerRegistry};

#[derive(Default)]
pub(super) struct EditorEventServiceState {
    pub(super) journal: EditorEventJournal,
    pub(super) listeners: EditorEventListenerRegistry,
    pub(super) next_event_id: u64,
    pub(super) next_sequence: u64,
    pub(super) revision: u64,
}
