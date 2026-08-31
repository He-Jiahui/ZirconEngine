mod discovery;
mod document_key;
mod error;
mod format;
mod limits;
mod model;
mod prepared;
mod reader;
mod store;
mod writer;

pub use discovery::{JournalDiscoveryEntry, JournalDiscoveryIssue, JournalDiscoveryReport};
pub use document_key::{JournalDocumentKey, JournalDocumentKeyError};
pub use error::{DurableJournalError, JournalRecordPreparationError};
pub use model::{
    DurableJournalEntry, JournalCompactionReport, JournalReadReport, JournalTailFault,
};
pub use prepared::PreparedJournalRecord;
pub use store::DurableJournal;
pub use writer::JournalWriter;

use super::TransactionJournal;
use limits::{MAX_HEADER_BYTES, MAX_JOURNAL_BYTES, MAX_JOURNAL_RECORDS, MAX_RECORD_BYTES};
use store::{JOURNAL_FORMAT_VERSION, JOURNAL_MAGIC};
