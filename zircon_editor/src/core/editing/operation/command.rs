use crate::core::editing::engine::{CommandBox, EditCommand, HistoryContextId, MergeMode};

pub struct OperationCommand {
    command: CommandBox,
    history: HistoryContextId,
    merge_mode: MergeMode,
}

impl OperationCommand {
    pub fn new(command: CommandBox, history: HistoryContextId) -> Self {
        Self {
            command,
            history,
            merge_mode: MergeMode::Disable,
        }
    }

    pub fn with_merge_mode(mut self, merge_mode: MergeMode) -> Self {
        self.merge_mode = merge_mode;
        self
    }

    pub fn command(&self) -> &dyn EditCommand {
        self.command.as_ref()
    }

    pub fn history(&self) -> HistoryContextId {
        self.history
    }

    pub fn merge_mode(&self) -> MergeMode {
        self.merge_mode
    }

    pub(crate) fn into_parts(self) -> (CommandBox, HistoryContextId, MergeMode) {
        (self.command, self.history, self.merge_mode)
    }
}
