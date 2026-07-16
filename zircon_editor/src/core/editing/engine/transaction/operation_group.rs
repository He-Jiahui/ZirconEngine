use super::{
    CommandBox, EditCommandError, EditorTransactionEngine, HistoryContextId, MergeMode,
    TransactionId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperationTransactionResult {
    pub transaction_id: TransactionId,
    pub group_open: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ActiveOperationGroup {
    pub(super) key: String,
    pub(super) history: HistoryContextId,
    pub(super) transaction: TransactionId,
}

impl EditorTransactionEngine {
    pub fn execute_operation(
        &self,
        label: impl Into<String>,
        history: HistoryContextId,
        operation_group: Option<&str>,
        merge_mode: MergeMode,
        command: CommandBox,
    ) -> Result<OperationTransactionResult, EditCommandError> {
        let label = label.into();
        let operation_group = operation_group.filter(|group| !group.is_empty());
        if let Some(group) = operation_group {
            let existing = self.lock_state().operation_group.clone();
            if let Some(existing) =
                existing.filter(|active| active.key == group && active.history == history)
            {
                self.set_merge_mode(existing.transaction, merge_mode);
                if let Err(error) = self.push(existing.transaction, command) {
                    self.lock_state().operation_group = None;
                    return Err(error);
                }
                return Ok(OperationTransactionResult {
                    transaction_id: existing.transaction,
                    group_open: true,
                });
            }

            self.flush_operation_group()?;
            let transaction = self.begin_transaction(label, history)?;
            self.set_merge_mode(transaction, merge_mode);
            self.lock_state().operation_group = Some(ActiveOperationGroup {
                key: group.to_string(),
                history,
                transaction,
            });
            if let Err(error) = self.push(transaction, command) {
                self.lock_state().operation_group = None;
                return Err(error);
            }
            return Ok(OperationTransactionResult {
                transaction_id: transaction,
                group_open: true,
            });
        }

        self.flush_operation_group()?;
        let transaction = self.begin_transaction(label, history)?;
        self.set_merge_mode(transaction, merge_mode);
        self.push(transaction, command)?;
        self.commit(transaction)?;
        Ok(OperationTransactionResult {
            transaction_id: transaction,
            group_open: false,
        })
    }

    pub fn flush_operation_group(&self) -> Result<Option<TransactionId>, EditCommandError> {
        let active = self.lock_state().operation_group.take();
        let Some(active) = active else {
            return Ok(None);
        };
        self.commit(active.transaction).map(Some)
    }
}
