use crate::core::editing::engine::EditCommandError;
use crate::ui::workbench::snapshot::TransactionHistorySnapshot;

use super::EditorState;

impl EditorState {
    /// 按当前world历史域查询；无活动域与查询失败分别返回None和Err。
    pub(crate) fn active_scene_transaction_history_snapshot(
        &self,
    ) -> Result<Option<TransactionHistorySnapshot>, EditCommandError> {
        self.active_scene_history_context()
            .map(|context| TransactionHistorySnapshot::query(self.transactions(), context))
            .transpose()
    }
}
