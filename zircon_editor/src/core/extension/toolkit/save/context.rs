use thiserror::Error;

use super::report::DocumentSaveGuarantee;
use super::source_write_authority::DocumentSourceWriteReceipt;
use super::SaveReason;

pub struct SaveCtx {
    reason: SaveReason,
    written_bytes: u64,
    source_write_guarantee: DocumentSaveGuarantee,
}

impl SaveCtx {
    pub(crate) const fn new(reason: SaveReason) -> Self {
        Self {
            reason,
            written_bytes: 0,
            source_write_guarantee: DocumentSaveGuarantee::default(),
        }
    }

    pub const fn reason(&self) -> SaveReason {
        self.reason
    }

    pub fn record_written_bytes(&mut self, bytes: u64) -> Result<(), SaveContextError> {
        self.written_bytes = self
            .written_bytes
            .checked_add(bytes)
            .ok_or(SaveContextError::WrittenByteCountOverflow)?;
        Ok(())
    }

    pub(crate) fn record_serialized_project_source_write(
        &mut self,
        bytes: u64,
        receipt: DocumentSourceWriteReceipt,
    ) -> Result<(), SaveContextError> {
        let _ = receipt;
        self.record_written_bytes(bytes)?;
        self.source_write_guarantee = DocumentSaveGuarantee::serialized_project_source();
        Ok(())
    }

    pub(crate) const fn written_bytes(&self) -> u64 {
        self.written_bytes
    }

    pub(in crate::core::extension::toolkit) const fn source_write_guarantee(
        &self,
    ) -> DocumentSaveGuarantee {
        self.source_write_guarantee
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SaveContextError {
    #[error("document save written-byte count overflowed")]
    WrittenByteCountOverflow,
}
