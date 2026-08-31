use serde::{Deserialize, Serialize};
use serde_json::Value;
use zircon_runtime_interface::serialization::{
    load_versioned, write_versioned, Format, LoadError, MigrateError, MigrationChain,
    MigrationStep, SchemaId, VersionedSchema, WriteError,
};

use crate::core::editing::selection::SelectionJournal;

use super::super::{EditCommandError, HistoryContextId, TransactionId, TransactionRecord};
use super::CommandJournalPayload;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransactionJournal {
    transaction: TransactionId,
    history: HistoryContextId,
    label: String,
    timestamp_frame: u64,
    participants: Vec<crate::core::editor_message::DocumentId>,
    selection_before: SelectionJournal,
    selection_after: SelectionJournal,
    significant: bool,
    commands: Vec<CommandJournalPayload>,
}

impl TransactionJournal {
    pub(crate) fn from_record(
        history: HistoryContextId,
        record: &TransactionRecord,
    ) -> Result<Self, TransactionJournalError> {
        if history.is_volatile() {
            return Err(TransactionJournalError::VolatileHistory { history });
        }
        let commands = record
            .commands
            .iter()
            .enumerate()
            .map(|(command_index, command)| {
                command
                    .journal_payload()
                    .map_err(|_| TransactionJournalError::UnsupportedCommand {
                        transaction: record.id,
                        command_index,
                        label: command.label().to_string(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            transaction: record.id,
            history,
            label: record.label.clone(),
            timestamp_frame: record.timestamp_frame,
            participants: record.participants.iter().copied().collect(),
            selection_before: record.selection_before.journal_projection(),
            selection_after: record.selection_after.journal_projection(),
            significant: record.significant,
            commands,
        })
    }

    pub fn validate(&self) -> Result<(), TransactionJournalValidationError> {
        if self.history.is_volatile() {
            return Err(TransactionJournalValidationError::VolatileHistory {
                history: self.history,
            });
        }
        Ok(())
    }

    /// Encodes the current transaction payload inside the shared editor schema envelope.
    pub fn encode(&self) -> Result<Vec<u8>, WriteError> {
        write_versioned(self, Format::Text)
    }

    /// Decodes only the current shared schema contract; retired raw payloads fail closed.
    pub fn decode(bytes: &[u8]) -> Result<Self, TransactionJournalReadError> {
        let journal = load_versioned::<Self>(bytes, Format::Text)
            .map_err(TransactionJournalReadError::Decode)?
            .value;
        journal
            .validate()
            .map_err(TransactionJournalReadError::Validation)?;
        Ok(journal)
    }

    pub const fn transaction(&self) -> TransactionId {
        self.transaction
    }

    pub const fn history(&self) -> HistoryContextId {
        self.history
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub const fn timestamp_frame(&self) -> u64 {
        self.timestamp_frame
    }

    pub fn participants(&self) -> &[crate::core::editor_message::DocumentId] {
        &self.participants
    }

    pub fn selection_before(&self) -> &SelectionJournal {
        &self.selection_before
    }

    pub fn selection_after(&self) -> &SelectionJournal {
        &self.selection_after
    }

    pub const fn significant(&self) -> bool {
        self.significant
    }

    pub fn commands(&self) -> &[CommandJournalPayload] {
        &self.commands
    }
}

impl VersionedSchema for TransactionJournal {
    const SCHEMA: SchemaId = SchemaId::new("zircon.editor.editing.transaction-journal");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<TransactionJournal> =
            MigrationChain::new(&[MigrationStep::new(0, reject_retired_transaction_journal)]);
        &MIGRATIONS
    }
}

fn reject_retired_transaction_journal(_value: Value) -> Result<Value, MigrateError> {
    Err(MigrateError::invalid_payload(
        "unversioned transaction journal payloads are retired",
    ))
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionJournalError {
    #[error("volatile history {history:?} cannot be persisted as a transaction journal")]
    VolatileHistory { history: HistoryContextId },
    #[error("transaction {transaction:?} does not exist in history {history:?}")]
    TransactionNotFound {
        history: HistoryContextId,
        transaction: TransactionId,
    },
    #[error("transaction {transaction:?} command {command_index} ({label}) cannot be journaled")]
    UnsupportedCommand {
        transaction: TransactionId,
        command_index: usize,
        label: String,
    },
    #[error("transaction journal query could not enter the engine")]
    Engine {
        #[source]
        source: EditCommandError,
    },
}

impl From<EditCommandError> for TransactionJournalError {
    fn from(source: EditCommandError) -> Self {
        Self::Engine { source }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TransactionJournalValidationError {
    #[error("volatile history {history:?} cannot be replayed as a durable transaction journal")]
    VolatileHistory { history: HistoryContextId },
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionJournalReadError {
    #[error("transaction journal payload could not be decoded")]
    Decode(#[source] LoadError),
    #[error("transaction journal payload violates engine invariants")]
    Validation(#[source] TransactionJournalValidationError),
}
