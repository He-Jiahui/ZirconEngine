use std::any::Any;

use serde_json::json;

use crate::core::editing::engine::{
    CommandExecutionError, CommandJournalPayload, EditCommand, EditCommandCodec,
    EditCommandCodecRegistry, EditContext, EditorTransactionEngine, HistoryContextId,
    JournalCodecDecodeError, JournalReplayError, MergeOutcome, TransactionJournalReplayer,
};
use crate::core::editor_message::DocumentId;

use super::fixture::FixtureContext;

#[test]
fn journal_replay_decodes_every_command_before_entering_the_target_history() {
    let source = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = source
        .begin("codec source", HistoryContextId::Global)
        .unwrap();
    scope.push(ReplayableCommand { delta: 9 }).unwrap();
    let transaction = scope.commit().unwrap();
    let journal = source
        .journal_transaction(HistoryContextId::Global, transaction)
        .unwrap();
    let mut codecs = EditCommandCodecRegistry::new();
    codecs.register(ReplayableCommandCodec).unwrap();

    let target = EditorTransactionEngine::new(FixtureContext::default());
    let replayer = TransactionJournalReplayer::new(&codecs);
    let target_history = HistoryContextId::Document(DocumentId::new(91));
    replayer.replay(&target, target_history, &journal).unwrap();

    assert_eq!(
        target
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(9)
    );
    assert_eq!(target.history_status(target_history).unwrap().len, 1);
}

#[test]
fn journal_replay_rejects_unknown_codecs_without_mutating_the_target_context() {
    let source = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = source
        .begin("unknown codec source", HistoryContextId::Global)
        .unwrap();
    scope.push(ReplayableCommand { delta: 4 }).unwrap();
    let transaction = scope.commit().unwrap();
    let journal = source
        .journal_transaction(HistoryContextId::Global, transaction)
        .unwrap();
    let target = EditorTransactionEngine::new(FixtureContext::default());
    let codecs = EditCommandCodecRegistry::new();

    let error = TransactionJournalReplayer::new(&codecs)
        .replay(&target, HistoryContextId::Global, &journal)
        .unwrap_err();
    assert!(matches!(
        error,
        JournalReplayError::Decode(
            crate::core::editing::engine::JournalCodecError::Unregistered { .. }
        )
    ));
    assert_eq!(
        target
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(0)
    );
    assert_eq!(
        target.history_status(HistoryContextId::Global).unwrap().len,
        0
    );
}

struct ReplayableCommand {
    delta: i32,
}

impl EditCommand for ReplayableCommand {
    fn label(&self) -> &str {
        "replayable command"
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let fixture = context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .expect("journal codec fixture context");
        fixture.value += self.delta;
        Ok(())
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let fixture = context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .expect("journal codec fixture context");
        fixture.value -= self.delta;
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
            "test.replayable_command",
            1,
            json!({ "delta": self.delta }),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct ReplayableCommandCodec;

impl EditCommandCodec for ReplayableCommandCodec {
    fn command_type(&self) -> &str {
        "test.replayable_command"
    }

    fn schema_version(&self) -> u16 {
        1
    }

    fn decode(
        &self,
        payload: &serde_json::Value,
    ) -> Result<Box<dyn EditCommand>, JournalCodecDecodeError> {
        let delta = payload
            .get("delta")
            .and_then(serde_json::Value::as_i64)
            .and_then(|delta| i32::try_from(delta).ok())
            .ok_or_else(|| {
                JournalCodecDecodeError::invalid_payload("delta must be a signed 32-bit integer")
            })?;
        Ok(Box::new(ReplayableCommand { delta }))
    }
}
