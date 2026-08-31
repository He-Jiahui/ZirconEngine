use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::{
    write_header, DurableJournalError, JournalDocumentKey, PreparedJournalRecord,
    MAX_JOURNAL_BYTES, MAX_RECORD_BYTES,
};

pub struct JournalWriter {
    path: PathBuf,
    file: File,
    next_sequence: u64,
    poisoned: bool,
}

impl JournalWriter {
    pub(super) fn create(
        path: PathBuf,
        document: &JournalDocumentKey,
    ) -> Result<Self, DurableJournalError> {
        Self::create_with_base(path, document, 0)
    }

    pub(super) fn create_with_base(
        path: PathBuf,
        document: &JournalDocumentKey,
        base_sequence: u64,
    ) -> Result<Self, DurableJournalError> {
        let mut file = open_new(&path)?;
        if let Err(error) = write_header(&mut file, document, base_sequence, &path) {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        let next_sequence = base_sequence
            .checked_add(1)
            .ok_or_else(|| DurableJournalError::SequenceExhausted { path: path.clone() })?;
        Ok(Self {
            path,
            file,
            next_sequence,
            poisoned: false,
        })
    }

    pub(super) fn open_existing(
        path: PathBuf,
        next_sequence: u64,
    ) -> Result<Self, DurableJournalError> {
        Ok(Self {
            file: open_append(&path)?,
            path,
            next_sequence,
            poisoned: false,
        })
    }

    /// Appends one record whose bytes and digest were frozen before writer admission.
    pub fn append_prepared(
        &mut self,
        prepared: PreparedJournalRecord,
    ) -> Result<u64, DurableJournalError> {
        if self.poisoned {
            return Err(DurableJournalError::WriterPoisoned {
                path: self.path.clone(),
            });
        }
        let payload = prepared.payload();
        if payload.len() > MAX_RECORD_BYTES {
            return Err(DurableJournalError::RecordTooLarge {
                path: self.path.clone(),
                bytes: payload.len(),
                maximum: MAX_RECORD_BYTES,
            });
        }
        let frame_bytes = 8_u64
            .saturating_add(4)
            .saturating_add(blake3::OUT_LEN as u64)
            .saturating_add(payload.len() as u64);
        let existing_bytes = self
            .file
            .metadata()
            .map_err(|source| DurableJournalError::Io {
                operation: "inspect durable journal before append",
                path: self.path.clone(),
                source,
            })?
            .len();
        let total_bytes = existing_bytes.checked_add(frame_bytes).ok_or_else(|| {
            DurableJournalError::FileTooLarge {
                path: self.path.clone(),
                bytes: u64::MAX,
                maximum: MAX_JOURNAL_BYTES,
            }
        })?;
        if total_bytes > MAX_JOURNAL_BYTES {
            return Err(DurableJournalError::FileTooLarge {
                path: self.path.clone(),
                bytes: total_bytes,
                maximum: MAX_JOURNAL_BYTES,
            });
        }
        let sequence = self.next_sequence;
        let next_sequence =
            sequence
                .checked_add(1)
                .ok_or_else(|| DurableJournalError::SequenceExhausted {
                    path: self.path.clone(),
                })?;
        let write_result = self
            .file
            .write_all(&sequence.to_le_bytes())
            .and_then(|()| self.file.write_all(&(payload.len() as u32).to_le_bytes()))
            .and_then(|()| self.file.write_all(prepared.digest()))
            .and_then(|()| self.file.write_all(&payload))
            .and_then(|()| self.file.sync_data());
        if let Err(source) = write_result {
            self.poisoned = true;
            return Err(DurableJournalError::Io {
                operation: "append durable journal record",
                path: self.path.clone(),
                source,
            });
        }
        self.next_sequence = next_sequence;
        Ok(sequence)
    }

    pub const fn next_sequence(&self) -> u64 {
        self.next_sequence
    }
}

fn open_append(path: &Path) -> Result<File, DurableJournalError> {
    OpenOptions::new()
        .read(true)
        .append(true)
        .open(path)
        .map_err(|source| DurableJournalError::Io {
            operation: "open durable journal",
            path: path.to_path_buf(),
            source,
        })
}

fn open_new(path: &Path) -> Result<File, DurableJournalError> {
    OpenOptions::new()
        .create_new(true)
        .read(true)
        .append(true)
        .open(path)
        .map_err(|source| DurableJournalError::Io {
            operation: "create durable journal",
            path: path.to_path_buf(),
            source,
        })
}
