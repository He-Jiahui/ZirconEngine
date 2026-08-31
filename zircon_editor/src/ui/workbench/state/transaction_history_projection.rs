use crate::core::editing::engine::EditCommandError;
use crate::ui::workbench::snapshot::TransactionHistorySnapshot;

use super::EditorState;

impl EditorState {
    pub(crate) fn active_scene_transaction_history_snapshot(
        &self,
    ) -> Result<Option<TransactionHistorySnapshot>, EditCommandError> {
        self.active_scene_history_context()
            .map(|context| TransactionHistorySnapshot::query(self.transactions(), context))
            .transpose()
    }
}
