use serde::{Deserialize, Serialize};

use crate::core::editing::selection::SelectionJournal;

use super::{EditCommandError, HistoryContextId, TransactionId, TransactionRecord};

pub const TRANSACTION_JOURNAL_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandJournalPayload {
    command_type: String,
    schema_version: u16,
    payload: serde_json::Value,
}

impl CommandJournalPayload {
    pub fn new(
        command_type: impl Into<String>,
        schema_version: u16,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            command_type: command_type.into(),
            schema_version,
            payload,
        }
    }

    pub fn command_type(&self) -> &str {
        &self.command_type
    }

    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub fn payload(&self) -> &serde_json::Value {
        &self.payload
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionJournal {
    schema_version: u16,
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
            schema_version: TRANSACTION_JOURNAL_SCHEMA_VERSION,
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

    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub fn validate_schema(&self) -> Result<(), TransactionJournalSchemaError> {
        if self.schema_version == TRANSACTION_JOURNAL_SCHEMA_VERSION {
            Ok(())
        } else {
            Err(TransactionJournalSchemaError::UnsupportedSchema {
                found: self.schema_version,
            })
        }
    }

    /// Decodes only a journal contract that this engine version can consume.
    pub fn decode(bytes: &[u8]) -> Result<Self, TransactionJournalReadError> {
        let journal = serde_json::from_slice(bytes).map_err(TransactionJournalReadError::Decode)?;
        journal
            .validate_schema()
            .map_err(TransactionJournalReadError::Schema)?;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandJournalUnavailable {
    label: String,
}

impl CommandJournalUnavailable {
    pub(crate) fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionJournalError {
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
pub enum TransactionJournalSchemaError {
    #[error("transaction journal schema {found} is not supported")]
    UnsupportedSchema { found: u16 },
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionJournalReadError {
    #[error("transaction journal payload could not be decoded")]
    Decode(#[source] serde_json::Error),
    #[error("transaction journal schema is incompatible")]
    Schema(#[source] TransactionJournalSchemaError),
}
