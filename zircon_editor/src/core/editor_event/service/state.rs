#[derive(Default)]
pub(super) struct EditorEventSequenceState {
    pub(super) next_event_id: u64,
    pub(super) next_sequence: u64,
    pub(super) revision: u64,
}
