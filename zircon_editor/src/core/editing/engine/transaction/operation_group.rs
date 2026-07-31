use std::sync::Arc;

use super::{
    CommandBox, EditCommandError, EditorTransactionEngine, HistoryContextId, MergeMode,
    TransactionId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperationTransactionResult {
    pub transaction_id: TransactionId,
    pub group_open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperationGroupPhase {
    Initializing,
    Open,
    Flushing,
}

impl OperationGroupPhase {
    const fn operation(self) -> &'static str {
        match self {
            Self::Initializing => "initialize operation group",
            Self::Open => "use operation group",
            Self::Flushing => "flush operation group",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct OperationGroupReservation(Arc<()>);

impl OperationGroupReservation {
    fn new() -> Self {
        Self(Arc::new(()))
    }

    fn same_identity(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Debug)]
pub(super) struct ActiveOperationGroup {
    pub(super) key: String,
    pub(super) history: HistoryContextId,
    pub(super) transaction: Option<TransactionId>,
    reservation: OperationGroupReservation,
    phase: OperationGroupPhase,
}

impl ActiveOperationGroup {
    pub(super) fn allows_begin(
        &self,
        history: HistoryContextId,
        reservation: Option<&OperationGroupReservation>,
    ) -> bool {
        self.history == history
            && self.transaction.is_none()
            && self.phase == OperationGroupPhase::Initializing
            && reservation.is_some_and(|reservation| self.reservation.same_identity(reservation))
    }

    pub(super) fn operation(&self) -> &'static str {
        self.phase.operation()
    }
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
            let (existing_transaction, blocked_by) = {
                let state = self.lock_state();
                match state.operation_group.as_ref() {
                    Some(active) if active.phase != OperationGroupPhase::Open => {
                        (None, Some(active.phase.operation()))
                    }
                    Some(active) if active.key == group && active.history == history => {
                        (active.transaction, None)
                    }
                    Some(_) | None => (None, None),
                }
            };
            if let Some(active) = blocked_by {
                return Err(EditCommandError::EngineBusy {
                    active,
                    requested: "execute operation group",
                });
            }
            if let Some(transaction) = existing_transaction {
                self.set_merge_mode(transaction, merge_mode);
                if let Err(error) = self.push(transaction, command) {
                    if !matches!(error, EditCommandError::EngineBusy { .. })
                        && !self.has_active_scope(transaction)
                    {
                        self.clear_operation_group_for_transaction(transaction);
                    }
                    return Err(error);
                }
                return Ok(OperationTransactionResult {
                    transaction_id: transaction,
                    group_open: true,
                });
            }

            self.flush_operation_group()?;
            let reservation = self.reserve_operation_group(group, history)?;
            let transaction = match self.begin_transaction(label, history, Some(&reservation)) {
                Ok(transaction) => transaction,
                Err(error) => {
                    self.clear_initializing_operation_group(group, history, None, &reservation);
                    return Err(error);
                }
            };
            let bound = {
                let mut state = self.lock_state();
                match state.operation_group.as_mut() {
                    Some(active)
                        if active.key == group
                            && active.history == history
                            && active.transaction.is_none()
                            && active.phase == OperationGroupPhase::Initializing =>
                    {
                        if active.reservation.same_identity(&reservation) {
                            active.transaction = Some(transaction);
                            true
                        } else {
                            false
                        }
                    }
                    Some(_) | None => false,
                }
            };
            if !bound {
                self.cleanup_initializing_operation_group(transaction)?;
                return Err(EditCommandError::InvariantViolation {
                    invariant: "operation group initialization reservation must remain owned",
                });
            }
            self.set_merge_mode(transaction, merge_mode);
            if let Err(error) = self.push(transaction, command) {
                let preserve_original = matches!(&error, EditCommandError::RollbackFailed { .. });
                let cleanup = self.cleanup_initializing_operation_group(transaction);
                if let Err(cleanup_error) = cleanup {
                    if !preserve_original {
                        return Err(cleanup_error);
                    }
                }
                return Err(error);
            }
            let mut state = self.lock_state();
            if let Some(active) = state
                .operation_group
                .as_mut()
                .filter(|active| active.transaction == Some(transaction))
            {
                active.phase = OperationGroupPhase::Open;
            }
            return Ok(OperationTransactionResult {
                transaction_id: transaction,
                group_open: true,
            });
        }

        self.flush_operation_group()?;
        let transaction = self.begin_transaction(label, history, None)?;
        self.set_merge_mode(transaction, merge_mode);
        self.push(transaction, command)?;
        self.commit(transaction)?;
        Ok(OperationTransactionResult {
            transaction_id: transaction,
            group_open: false,
        })
    }

    pub fn flush_operation_group(&self) -> Result<Option<TransactionId>, EditCommandError> {
        let active = {
            let mut state = self.lock_state();
            let Some(active) = state.operation_group.as_mut() else {
                return Ok(None);
            };
            if active.phase != OperationGroupPhase::Open {
                return Err(EditCommandError::EngineBusy {
                    active: active.phase.operation(),
                    requested: "flush operation group",
                });
            }
            let transaction = active
                .transaction
                .ok_or(EditCommandError::InvariantViolation {
                    invariant: "an open operation group must own a transaction",
                })?;
            active.phase = OperationGroupPhase::Flushing;
            transaction
        };
        match self.commit(active) {
            Ok(transaction) => {
                let mut state = self.lock_state();
                if state
                    .operation_group
                    .as_ref()
                    .is_some_and(|current| current.transaction == Some(active))
                {
                    state.operation_group = None;
                }
                Ok(Some(transaction))
            }
            Err(error) => {
                let preserve = matches!(error, EditCommandError::EngineBusy { .. })
                    || self.has_active_scope(active);
                let mut state = self.lock_state();
                if state
                    .operation_group
                    .as_ref()
                    .is_some_and(|current| current.transaction == Some(active))
                {
                    if preserve {
                        if let Some(current) = state.operation_group.as_mut() {
                            current.phase = OperationGroupPhase::Open;
                        }
                    } else {
                        state.operation_group = None;
                    }
                }
                Err(error)
            }
        }
    }

    fn cleanup_initializing_operation_group(
        &self,
        transaction: TransactionId,
    ) -> Result<(), EditCommandError> {
        let cleanup = loop {
            if !self.has_active_scope(transaction) {
                break Ok(());
            }
            match self.cancel(transaction) {
                Err(EditCommandError::EngineBusy { .. }) => self.wait_for_operation(),
                result => break result,
            }
        };
        let mut state = self.lock_state();
        if state
            .operation_group
            .as_ref()
            .is_some_and(|active| active.transaction == Some(transaction))
        {
            state.operation_group = None;
        }
        cleanup
    }

    fn clear_operation_group_for_transaction(&self, transaction: TransactionId) {
        let mut state = self.lock_state();
        if state
            .operation_group
            .as_ref()
            .is_some_and(|active| active.transaction == Some(transaction))
        {
            state.operation_group = None;
        }
    }

    fn reserve_operation_group(
        &self,
        key: &str,
        history: HistoryContextId,
    ) -> Result<OperationGroupReservation, EditCommandError> {
        let reservation = OperationGroupReservation::new();
        let mut state = self.lock_state();
        if let Some(active) = state.operation_group.as_ref() {
            return Err(EditCommandError::EngineBusy {
                active: active.phase.operation(),
                requested: "execute operation group",
            });
        }
        state.operation_group = Some(ActiveOperationGroup {
            key: key.to_string(),
            history,
            transaction: None,
            reservation: reservation.clone(),
            phase: OperationGroupPhase::Initializing,
        });
        Ok(reservation)
    }

    fn clear_initializing_operation_group(
        &self,
        key: &str,
        history: HistoryContextId,
        transaction: Option<TransactionId>,
        reservation: &OperationGroupReservation,
    ) {
        let mut state = self.lock_state();
        if state.operation_group.as_ref().is_some_and(|active| {
            active.key == key
                && active.history == history
                && active.transaction == transaction
                && active.reservation.same_identity(reservation)
                && active.phase == OperationGroupPhase::Initializing
        }) {
            state.operation_group = None;
        }
    }
}

#[cfg(test)]
mod performance_source_guards {
    use std::any::Any;

    use crate::core::editing::engine::{
        CommandExecutionError, EditCommand, EditContext, SelectionSnapshot,
    };
    use crate::core::gateway::EditorRuntimeGatewayHandle;

    use super::*;

    struct TestContext {
        gateway: EditorRuntimeGatewayHandle,
    }

    impl Default for TestContext {
        fn default() -> Self {
            Self {
                gateway: EditorRuntimeGatewayHandle::detached(),
            }
        }
    }

    impl EditContext for TestContext {
        fn runtime_gateway(&self) -> &EditorRuntimeGatewayHandle {
            &self.gateway
        }

        fn selection_snapshot(&self) -> SelectionSnapshot {
            SelectionSnapshot::default()
        }

        fn restore_selection(
            &mut self,
            _snapshot: &SelectionSnapshot,
        ) -> Result<(), EditCommandError> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    struct TestCommand;

    impl EditCommand for TestCommand {
        fn label(&self) -> &str {
            "operation group state test"
        }

        fn apply(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            Ok(())
        }

        fn revert(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn continuing_an_operation_group_does_not_clone_its_stable_key() {
        let source = include_str!("operation_group.rs");
        let cloned_group = ["operation_group", ".clone()"].concat();

        assert!(!source.contains(&cloned_group));
        assert!(!source.contains("active.clone()"));
    }

    #[test]
    fn unowned_begin_cannot_cross_live_operation_group_reservation() {
        let engine = EditorTransactionEngine::new(TestContext::default());
        assert_eq!(engine.flush_operation_group().unwrap(), None);
        let reservation = engine
            .reserve_operation_group("reserved", HistoryContextId::Global)
            .unwrap();

        assert!(matches!(
            engine.begin_transaction("stale caller", HistoryContextId::Global, None),
            Err(EditCommandError::EngineBusy {
                active: "initialize operation group",
                ..
            })
        ));

        let transaction = engine
            .begin_transaction(
                "reservation owner",
                HistoryContextId::Global,
                Some(&reservation),
            )
            .unwrap();
        engine.cancel(transaction).unwrap();
        engine.clear_initializing_operation_group(
            "reserved",
            HistoryContextId::Global,
            None,
            &reservation,
        );
    }

    #[test]
    fn stale_operation_group_cleanup_preserves_successor() {
        let engine = EditorTransactionEngine::new(TestContext::default());
        let first = engine
            .execute_operation(
                "first",
                HistoryContextId::Global,
                Some("first"),
                MergeMode::Disable,
                Box::new(TestCommand),
            )
            .unwrap();
        assert_eq!(
            engine.flush_operation_group().unwrap(),
            Some(first.transaction_id)
        );
        let successor = engine
            .execute_operation(
                "successor",
                HistoryContextId::Global,
                Some("successor"),
                MergeMode::Disable,
                Box::new(TestCommand),
            )
            .unwrap();

        engine.clear_operation_group_for_transaction(first.transaction_id);

        assert_eq!(
            engine.flush_operation_group().unwrap(),
            Some(successor.transaction_id)
        );
    }
}
