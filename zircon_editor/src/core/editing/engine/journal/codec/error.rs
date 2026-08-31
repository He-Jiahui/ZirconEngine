use super::super::TransactionJournalValidationError;
use crate::core::editing::engine::EditCommandError;

#[derive(Clone, Debug, thiserror::Error)]
#[error("journal command payload is invalid: {message}")]
pub struct JournalCodecDecodeError {
    message: String,
}

impl JournalCodecDecodeError {
    pub fn invalid_payload(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JournalCodecError {
    #[error("journal codec {command_type}@{schema_version} is already registered")]
    Duplicate {
        command_type: String,
        schema_version: u16,
    },
    #[error("journal codec command type cannot be empty")]
    EmptyCommandType,
    #[error("journal command {command_type}@{schema_version} has no registered codec")]
    Unregistered {
        command_type: String,
        schema_version: u16,
    },
    #[error("journal command {command_type}@{schema_version} could not be decoded")]
    Decode {
        command_type: String,
        schema_version: u16,
        #[source]
        source: JournalCodecDecodeError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum JournalReplayError {
    #[error("journal payload violates engine invariants")]
    JournalValidation(#[source] TransactionJournalValidationError),
    #[error("journal command decoding failed")]
    Decode(#[source] JournalCodecError),
    #[error("journal replay could not enter the transaction engine")]
    Engine(#[source] EditCommandError),
}
