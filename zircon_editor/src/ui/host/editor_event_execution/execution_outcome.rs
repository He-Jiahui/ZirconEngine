use crate::core::editor_event::EditorEventEffect;

pub(crate) struct ExecutionOutcome {
    pub(crate) changed: bool,
    pub(crate) effects: Vec<EditorEventEffect>,
}

impl ExecutionOutcome {
    pub(crate) fn changed(&self) -> bool {
        self.changed
    }

    pub(crate) fn effects(&self) -> &[EditorEventEffect] {
        &self.effects
    }
}
