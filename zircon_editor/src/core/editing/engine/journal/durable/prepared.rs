use super::limits::MAX_RECORD_BYTES;
use super::JournalRecordPreparationError;
use crate::core::editing::engine::{TransactionId, TransactionJournal};

/// Immutable, size-validated payload ready for one durable journal frame.
///
/// The record owns the JSON bytes and digest, so a later bounded writer job never needs to retain
/// an engine-owned command or encode the same transaction a second time.
#[derive(Debug)]
pub struct PreparedJournalRecord {
    transaction: TransactionId,
    payload: Vec<u8>,
    digest: [u8; blake3::OUT_LEN],
}

impl PreparedJournalRecord {
    pub fn prepare(
        transaction: &TransactionJournal,
    ) -> Result<Self, JournalRecordPreparationError> {
        let payload = transaction
            .encode()
            .map_err(JournalRecordPreparationError::Encode)?;
        if payload.len() > MAX_RECORD_BYTES {
            return Err(JournalRecordPreparationError::RecordTooLarge {
                bytes: payload.len(),
                maximum: MAX_RECORD_BYTES,
            });
        }
        let digest = *blake3::hash(&payload).as_bytes();
        Ok(Self {
            transaction: transaction.transaction(),
            payload,
            digest,
        })
    }

    pub const fn transaction(&self) -> TransactionId {
        self.transaction
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub const fn digest(&self) -> &[u8; blake3::OUT_LEN] {
        &self.digest
    }
}
