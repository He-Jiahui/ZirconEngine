use super::super::TransactionJournal;
use super::{EditCommandCodecRegistry, JournalReplayError};
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId, TransactionId};

pub struct TransactionJournalReplayer<'codecs> {
    codecs: &'codecs EditCommandCodecRegistry,
}

impl<'codecs> TransactionJournalReplayer<'codecs> {
    pub const fn new(codecs: &'codecs EditCommandCodecRegistry) -> Self {
        Self { codecs }
    }

    /// Replays into the caller's live history context, never the persisted session-local id.
    pub fn replay(
        &self,
        engine: &EditorTransactionEngine,
        target_history: HistoryContextId,
        journal: &TransactionJournal,
    ) -> Result<TransactionId, JournalReplayError> {
        journal
            .validate()
            .map_err(JournalReplayError::JournalValidation)?;
        // Decode the whole record before opening a scope so a bad payload cannot partially mutate it.
        let commands = self
            .codecs
            .decode_all(journal.commands())
            .map_err(JournalReplayError::Decode)?;
        let mut scope = engine
            .begin(journal.label(), target_history)
            .map_err(JournalReplayError::Engine)?;
        for command in commands {
            scope
                .push_boxed(command)
                .map_err(JournalReplayError::Engine)?;
        }
        scope.commit().map_err(JournalReplayError::Engine)
    }
}
