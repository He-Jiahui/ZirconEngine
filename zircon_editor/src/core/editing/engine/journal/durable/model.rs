use std::path::{Path, PathBuf};

use crate::core::editing::engine::{TransactionJournal, TransactionJournalReadError};

use super::{DurableJournalError, JournalDocumentKey};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurableJournalEntry {
    pub(super) sequence: u64,
    pub(super) transaction: TransactionJournal,
}

impl DurableJournalEntry {
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn transaction(&self) -> &TransactionJournal {
        &self.transaction
    }
}

#[derive(Debug)]
pub struct JournalReadReport {
    pub(super) base_sequence: u64,
    pub(super) entries: Vec<DurableJournalEntry>,
    pub(super) tail_fault: Option<JournalTailFault>,
}

impl JournalReadReport {
    pub const fn base_sequence(&self) -> u64 {
        self.base_sequence
    }

    pub fn entries(&self) -> &[DurableJournalEntry] {
        &self.entries
    }

    pub fn tail_fault(&self) -> Option<&JournalTailFault> {
        self.tail_fault.as_ref()
    }

    pub(super) fn take_tail_fault(&mut self) -> Option<JournalTailFault> {
        self.tail_fault.take()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalCompactionReport {
    pub(super) covered_through: u64,
    pub(super) discarded_entries: usize,
    pub(super) retained_entries: usize,
}

impl JournalCompactionReport {
    pub const fn covered_through(&self) -> u64 {
        self.covered_through
    }

    pub const fn discarded_entries(&self) -> usize {
        self.discarded_entries
    }

    pub const fn retained_entries(&self) -> usize {
        self.retained_entries
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JournalTailFault {
    #[error("journal frame is truncated")]
    TruncatedFrame,
    #[error("journal record {sequence} is {length} bytes, exceeding the record limit")]
    RecordTooLarge { sequence: u64, length: usize },
    #[error("journal record {sequence} checksum does not match its payload")]
    ChecksumMismatch { sequence: u64 },
    #[error("journal record {sequence} is not a readable transaction")]
    InvalidTransaction {
        sequence: u64,
        #[source]
        source: TransactionJournalReadError,
    },
    #[error("journal record {sequence} does not continue the committed sequence")]
    NonContiguousSequence { sequence: u64 },
    #[error("journal record count exceeds the configured limit")]
    RecordLimitExceeded,
}

/// One discovered journal whose header and durable frames passed the recovery reader boundary.
#[derive(Debug)]
pub struct JournalDiscoveryEntry {
    document: JournalDocumentKey,
    path: PathBuf,
    report: JournalReadReport,
}

impl JournalDiscoveryEntry {
    pub(super) fn new(
        document: JournalDocumentKey,
        path: PathBuf,
        report: JournalReadReport,
    ) -> Self {
        Self {
            document,
            path,
            report,
        }
    }

    pub fn document(&self) -> &JournalDocumentKey {
        &self.document
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn report(&self) -> &JournalReadReport {
        &self.report
    }
}

/// One journal directory that recovery could not safely interpret.
#[derive(Debug)]
pub enum JournalDiscoveryIssue {
    Journal {
        path: PathBuf,
        error: DurableJournalError,
    },
    DirectoryKeyMismatch {
        path: PathBuf,
        expected_key: String,
        actual_key: String,
    },
}

impl JournalDiscoveryIssue {
    pub fn path(&self) -> &Path {
        match self {
            Self::Journal { path, .. } | Self::DirectoryKeyMismatch { path, .. } => path,
        }
    }

    pub fn error(&self) -> Option<&DurableJournalError> {
        match self {
            Self::Journal { error, .. } => Some(error),
            Self::DirectoryKeyMismatch { .. } => None,
        }
    }

    pub fn directory_key_mismatch(&self) -> Option<(&str, &str)> {
        match self {
            Self::DirectoryKeyMismatch {
                expected_key,
                actual_key,
                ..
            } => Some((expected_key, actual_key)),
            Self::Journal { .. } => None,
        }
    }
}

/// A per-entry isolated durable journal scan for startup recovery selection.
#[derive(Debug, Default)]
pub struct JournalDiscoveryReport {
    entries: Vec<JournalDiscoveryEntry>,
    issues: Vec<JournalDiscoveryIssue>,
}

impl JournalDiscoveryReport {
    pub(super) fn new(
        entries: Vec<JournalDiscoveryEntry>,
        issues: Vec<JournalDiscoveryIssue>,
    ) -> Self {
        Self { entries, issues }
    }

    pub fn entries(&self) -> &[JournalDiscoveryEntry] {
        &self.entries
    }

    pub fn issues(&self) -> &[JournalDiscoveryIssue] {
        &self.issues
    }
}
