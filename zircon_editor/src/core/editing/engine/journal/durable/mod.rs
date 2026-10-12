#[path = "discovery.rs"]
mod discovery;
#[path = "document_key.rs"]
mod document_key;
#[path = "error.rs"]
mod error;
#[path = "format.rs"]
mod format;
#[path = "limits.rs"]
mod limits;
#[path = "model.rs"]
mod model;
#[path = "prepared.rs"]
mod prepared;
#[path = "reader.rs"]
mod reader;
#[path = "store.rs"]
mod store;
#[path = "writer.rs"]
mod writer;

pub use document_key::{JournalDocumentKey, JournalDocumentKeyError};
pub use error::{DurableJournalError, JournalRecordPreparationError};
pub use model::{
    DurableJournalEntry, JournalCompactionReport, JournalDiscoveryEntry, JournalDiscoveryIssue,
    JournalDiscoveryReport, JournalReadReport, JournalTailFault,
};
pub use prepared::PreparedJournalRecord;
pub use store::DurableJournal;
pub use writer::JournalWriter;

use super::TransactionJournal;
use limits::{MAX_HEADER_BYTES, MAX_JOURNAL_BYTES, MAX_JOURNAL_RECORDS, MAX_RECORD_BYTES};
use store::{JOURNAL_FORMAT_VERSION, JOURNAL_MAGIC};
