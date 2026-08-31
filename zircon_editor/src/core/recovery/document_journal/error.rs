use std::path::PathBuf;

use thiserror::Error;

use crate::core::editing::engine::{
    DurableJournalError, JournalDocumentKeyError, JournalRecordPreparationError, TransactionId,
    TransactionJournalError,
};

#[derive(Debug, Error)]
pub enum DocumentJournalCoordinatorError {
    #[error(
        "document journal coordinator for project root {existing_root} cannot be reused for {requested_root}"
    )]
    ProjectRootConflict {
        existing_root: PathBuf,
        requested_root: PathBuf,
    },
    #[error(
        "document journal source {source_path} is outside the coordinator project root {project_root}"
    )]
    SourceOutsideProject {
        project_root: PathBuf,
        source_path: PathBuf,
    },
    #[error("document journal source path is invalid")]
    DocumentKey(#[source] JournalDocumentKeyError),
    #[error(
        "document journal binding for document {document} cannot change from {bound_source} to {requested_source}"
    )]
    BindingConflict {
        document: u64,
        bound_source: PathBuf,
        requested_source: PathBuf,
    },
    #[error("document {document} has no active durable journal binding")]
    DocumentNotBound { document: u64 },
    #[error("document {document} has an active durable journal binding without a writer")]
    WriterUnavailable { document: u64 },
    #[error("could not serialize committed transaction {transaction} for document {document}")]
    Transaction {
        document: u64,
        transaction: u64,
        #[source]
        source: TransactionJournalError,
    },
    #[error("could not prepare committed transaction {transaction} for document {document}")]
    PreparedRecord {
        document: u64,
        transaction: u64,
        #[source]
        source: JournalRecordPreparationError,
    },
    #[error("durable journal operation failed for document {document}")]
    Durable {
        document: u64,
        #[source]
        source: DurableJournalError,
    },
}

impl DocumentJournalCoordinatorError {
    pub(super) fn transaction(
        document: u64,
        transaction: TransactionId,
        source: TransactionJournalError,
    ) -> Self {
        Self::Transaction {
            document,
            transaction: transaction.raw(),
            source,
        }
    }

    pub(super) fn durable(document: u64, source: DurableJournalError) -> Self {
        Self::Durable { document, source }
    }

    pub(super) fn prepared(
        document: u64,
        transaction: TransactionId,
        source: JournalRecordPreparationError,
    ) -> Self {
        Self::PreparedRecord {
            document,
            transaction: transaction.raw(),
            source,
        }
    }
}
