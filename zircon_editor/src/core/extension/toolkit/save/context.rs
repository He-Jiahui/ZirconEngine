use thiserror::Error;

use super::SaveReason;

pub struct SaveCtx {
    reason: SaveReason,
    written_bytes: u64,
}

impl SaveCtx {
    pub(crate) const fn new(reason: SaveReason) -> Self {
        Self {
            reason,
            written_bytes: 0,
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

    pub(crate) const fn written_bytes(&self) -> u64 {
        self.written_bytes
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SaveContextError {
    #[error("document save written-byte count overflowed")]
    WrittenByteCountOverflow,
}
