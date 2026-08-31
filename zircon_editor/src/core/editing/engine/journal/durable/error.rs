use std::io;
use std::path::PathBuf;

use super::JournalTailFault;

#[derive(Debug, thiserror::Error)]
pub enum JournalRecordPreparationError {
    #[error("journal record could not be encoded")]
    Encode(#[source] zircon_runtime_interface::serialization::WriteError),
    #[error("journal record is {bytes} bytes, exceeding the {maximum}-byte limit")]
    RecordTooLarge { bytes: usize, maximum: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum DurableJournalError {
    #[error("journal I/O failed during {operation} at {path}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("journal root cannot derive a parent directory: {path}")]
    InvalidRoot { path: PathBuf },
    #[error("journal header at {path} could not be encoded")]
    HeaderEncode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("journal format at {path} is unsupported")]
    UnsupportedFormat { path: PathBuf },
    #[error("journal header at {path} is malformed")]
    InvalidHeader { path: PathBuf },
    #[error("journal at {path} is not a regular file")]
    UnexpectedFileType { path: PathBuf },
    #[error("journal at {path} does not belong to the requested document")]
    DocumentMismatch { path: PathBuf },
    #[error("journal at {path} has an unreadable tail: {tail}")]
    UnreadableTail {
        path: PathBuf,
        #[source]
        tail: JournalTailFault,
    },
    #[error("journal writer at {path} cannot accept appends after a failed durable write")]
    WriterPoisoned { path: PathBuf },
    #[error("journal at {path} is {bytes} bytes, exceeding the {maximum}-byte read limit")]
    FileTooLarge {
        path: PathBuf,
        bytes: u64,
        maximum: u64,
    },
    #[error("journal record at {path} is {bytes} bytes, exceeding the {maximum}-byte limit")]
    RecordTooLarge {
        path: PathBuf,
        bytes: usize,
        maximum: usize,
    },
    #[error("journal record at {path} could not be prepared")]
    Preparation {
        path: PathBuf,
        #[source]
        source: JournalRecordPreparationError,
    },
    #[error(
        "checkpoint sequence {requested} is outside journal range {base_sequence}..={last_sequence} at {path}"
    )]
    CheckpointOutOfRange {
        path: PathBuf,
        requested: u64,
        base_sequence: u64,
        last_sequence: u64,
    },
    #[error("journal compaction failed to preserve sequence {expected} at {path}")]
    CompactionSequenceMismatch { path: PathBuf, expected: u64 },
    #[error("journal sequence space is exhausted at {path}")]
    SequenceExhausted { path: PathBuf },
}
