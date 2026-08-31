use std::any::Any;

use serde_json::json;
use zircon_runtime_interface::serialization::{LoadError, VersionedSchema};

use crate::core::editing::engine::{
    CommandExecutionError, CommandJournalPayload, EditCommand, EditContext,
    EditorTransactionEngine, HistoryContextId, MergeOutcome, TransactionJournal,
    TransactionJournalError, TransactionJournalReadError,
};
use crate::core::editing::selection::SelectionJournal;

use super::fixture::FixtureContext;

#[test]
fn transaction_journal_round_trips_typed_metadata_and_command_payload() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    engine
        .with_context_mut::<FixtureContext, _>(|context| context.selection = 7)
        .unwrap();
    let mut scope = engine
        .begin("journal edit", HistoryContextId::Global)
        .unwrap();
    scope.push(JournalCommand).unwrap();
    let transaction = scope.commit().unwrap();

    let journal = engine
        .journal_transaction(HistoryContextId::Global, transaction)
        .unwrap();
    let encoded = journal.encode().unwrap();
    let decoded = TransactionJournal::decode(&encoded).unwrap();

    assert_eq!(decoded, journal);
    let envelope: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(
        envelope["$zircon"]["header"]["schema_id"],
        TransactionJournal::SCHEMA.as_str()
    );
    assert_eq!(
        envelope["$zircon"]["header"]["schema_version"],
        TransactionJournal::VERSION
    );
    assert_eq!(journal.transaction(), transaction);
    assert_eq!(journal.history(), HistoryContextId::Global);
    assert_eq!(journal.label(), "journal edit");
    assert_eq!(journal.timestamp_frame(), 0);
    assert_eq!(
        journal.selection_before(),
        &SelectionJournal::FixtureValue {
            generation: 7,
            value: 7,
        }
    );
    assert_eq!(journal.selection_after(), journal.selection_before());
    assert_eq!(
        journal.commands(),
        &[CommandJournalPayload::new(
            "test.journal_command",
            1,
            json!({ "delta": 1 })
        )]
    );

    engine.undo(HistoryContextId::Global).unwrap();
    assert_eq!(
        engine
            .journal_transaction(HistoryContextId::Global, transaction)
            .unwrap(),
        journal
    );
    engine.redo(HistoryContextId::Global).unwrap();
    assert_eq!(
        engine
            .journal_transaction(HistoryContextId::Global, transaction)
            .unwrap(),
        journal
    );
}

#[test]
fn transaction_journal_rejects_future_schema_version() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("versioned journal", HistoryContextId::Global)
        .unwrap();
    scope.push(JournalCommand).unwrap();
    let transaction = scope.commit().unwrap();
    let journal = engine
        .journal_transaction(HistoryContextId::Global, transaction)
        .unwrap();
    let mut encoded: serde_json::Value =
        serde_json::from_slice(&journal.encode().unwrap()).unwrap();
    let unsupported_version = TransactionJournal::VERSION + 1;
    encoded["$zircon"]["header"]["schema_version"] = json!(unsupported_version);

    let encoded = serde_json::to_vec(&encoded).unwrap();
    assert!(matches!(
        TransactionJournal::decode(&encoded).unwrap_err(),
        TransactionJournalReadError::Decode(LoadError::FutureVersion { found, .. })
            if found == unsupported_version
    ));
}

#[test]
fn transaction_journal_rejects_the_retired_private_version_payload() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("retired journal", HistoryContextId::Global)
        .unwrap();
    scope.push(JournalCommand).unwrap();
    let transaction = scope.commit().unwrap();
    let journal = engine
        .journal_transaction(HistoryContextId::Global, transaction)
        .unwrap();
    let mut retired = serde_json::to_value(journal).unwrap();
    retired["schema_version"] = json!(1);

    assert!(matches!(
        TransactionJournal::decode(&serde_json::to_vec(&retired).unwrap()),
        Err(TransactionJournalReadError::Decode(LoadError::Migration(_)))
    ));
}

#[test]
fn transaction_journal_rejects_retired_fields_inside_the_current_shell() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("retired field", HistoryContextId::Global)
        .unwrap();
    scope.push(JournalCommand).unwrap();
    let transaction = scope.commit().unwrap();
    let journal = engine
        .journal_transaction(HistoryContextId::Global, transaction)
        .unwrap();
    let mut encoded: serde_json::Value =
        serde_json::from_slice(&journal.encode().unwrap()).unwrap();
    encoded["$zircon"]["payload"]["schema_version"] = json!(1);

    assert!(matches!(
        TransactionJournal::decode(&serde_json::to_vec(&encoded).unwrap()),
        Err(TransactionJournalReadError::Decode(
            LoadError::PayloadDecode { .. }
        ))
    ));
}

#[test]
fn transaction_journal_rejects_unsupported_commands_with_typed_context() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("unsupported journal", HistoryContextId::Global)
        .unwrap();
    scope.push(UnsupportedCommand).unwrap();
    let transaction = scope.commit().unwrap();

    let error = engine
        .journal_transaction(HistoryContextId::Global, transaction)
        .unwrap_err();
    assert!(matches!(
        error,
        TransactionJournalError::UnsupportedCommand {
            transaction: actual_transaction,
            command_index: 0,
            label,
        } if actual_transaction == transaction && label == "unsupported journal"
    ));
}

struct JournalCommand;

impl EditCommand for JournalCommand {
    fn label(&self) -> &str {
        "journal command"
    }

    fn apply(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Ok(())
    }

    fn revert(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Ok(())
    }

    fn try_merge(&mut self, _next: &dyn EditCommand) -> MergeOutcome {
        MergeOutcome::Reject
    }

    fn journal_payload(
        &self,
    ) -> Result<CommandJournalPayload, crate::core::editing::engine::CommandJournalUnavailable>
    {
        Ok(CommandJournalPayload::new(
            "test.journal_command",
            1,
            json!({ "delta": 1 }),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct UnsupportedCommand;

impl EditCommand for UnsupportedCommand {
    fn label(&self) -> &str {
        "unsupported journal"
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
