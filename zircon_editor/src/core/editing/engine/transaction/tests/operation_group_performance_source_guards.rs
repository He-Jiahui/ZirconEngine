use std::any::Any;

use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditContext, EditWorldRoute, SelectionSnapshot,
};
use crate::core::play::WorldDomain;

use super::*;

#[derive(Default)]
struct TestContext;

impl EditContext for TestContext {
    fn capture_world_route(
        &self,
        world_domain: WorldDomain,
    ) -> Result<EditWorldRoute, EditCommandError> {
        Ok(EditWorldRoute::logical(world_domain))
    }

    fn activate_world_route(&mut self, _route: &EditWorldRoute) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn retire_world_route(&mut self, _world_domain: WorldDomain) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }

    fn restore_selection(&mut self, _snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
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
    let source = include_str!("../operation_group.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("operation-group production source");
    let cloned_group = ["operation_group", ".clone()"].concat();

    assert!(!production.contains(&cloned_group));
    assert!(!production.contains("active.clone()"));
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

#[test]
fn initializing_operation_group_observation_is_busy_without_finalization() {
    let engine = EditorTransactionEngine::new(TestContext::default());
    let history = HistoryContextId::Global;
    let reservation = engine
        .reserve_operation_group("initializing", history)
        .unwrap();
    assert!(matches!(
        engine.dirty_history_decision_token(history),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.history_status(history),
        Err(EditCommandError::EngineBusy {
            active: "initialize operation group",
            ..
        })
    ));
    assert!(matches!(
        engine.is_dirty(history),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.history_details(history, None, 1),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.history_generation_snapshot(history),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.dirty_states_since(None),
        Err(EditCommandError::EngineBusy { .. })
    ));
    engine.clear_initializing_operation_group("initializing", history, None, &reservation);
    assert_eq!(engine.history_status(history).unwrap().len, 0);
}

#[test]
fn flushing_operation_group_observation_is_busy_without_retiring_owner() {
    let engine = EditorTransactionEngine::new(TestContext::default());
    let history = HistoryContextId::Global;
    let result = engine
        .execute_operation(
            "pending",
            history,
            Some("pending"),
            MergeMode::Disable,
            Box::new(TestCommand),
        )
        .unwrap();
    engine.lock_state().operation_group.as_mut().unwrap().phase = OperationGroupPhase::Flushing;
    assert!(matches!(
        engine.dirty_history_decision_token(history),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.history_status(history),
        Err(EditCommandError::EngineBusy {
            active: "flush operation group",
            ..
        })
    ));
    assert!(matches!(
        engine.is_dirty(history),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.history_details(history, None, 1),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.history_generation_snapshot(history),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.dirty_states_since(None),
        Err(EditCommandError::EngineBusy { .. })
    ));
    engine.lock_state().operation_group.as_mut().unwrap().phase = OperationGroupPhase::Open;
    assert_eq!(engine.history_status(history).unwrap().len, 0);
    assert_eq!(
        engine.flush_operation_group().unwrap(),
        Some(result.transaction_id)
    );
    assert_eq!(engine.history_status(history).unwrap().len, 1);
}
