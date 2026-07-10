use crate::core::editor_event::{EditorEventId, EditorEventSequence};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EditorEventStamp {
    pub(crate) event_id: EditorEventId,
    pub(crate) sequence: EditorEventSequence,
    pub(crate) before_revision: u64,
    pub(crate) after_revision: u64,
}
