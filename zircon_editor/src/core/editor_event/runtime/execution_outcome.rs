use crate::core::editor_event::EditorEventEffect;

pub(super) struct ExecutionOutcome {
    pub(super) changed: bool,
    pub(super) effects: Vec<EditorEventEffect>,
}
