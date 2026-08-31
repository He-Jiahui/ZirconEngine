//! WAL validation, torn-tail classification, and safe recovery truncation.

use std::fs::OpenOptions;
use std::io;
use std::path::Path;

use super::super::error::{DurableTransactionError, TransactionPhase};
use super::super::schema::TransactionJournal;
use super::frame_codec::{MAX_JOURNAL_BYTES, TransitionAppend, decode_frame, parse_toml_frame};

#[cfg(any(test, feature = "test-support"))]
pub(in crate::io::transaction) fn decode_journal(
    path: &Path,
    bytes: &[u8],
) -> Result<TransactionJournal, DurableTransactionError> {
    decode_journal_with_valid_len(path, bytes).map(|(journal, _)| journal)
}

pub(in crate::io::transaction) fn decode_journal_with_valid_len(
    path: &Path,
    bytes: &[u8],
) -> Result<(TransactionJournal, usize), DurableTransactionError> {
    if bytes.len() > MAX_JOURNAL_BYTES {
        return Err(DurableTransactionError::invalid(
            path,
            "durable transaction journal exceeds its bounded size",
        ));
    }
    let mut offset = 0;
    let intent = decode_frame(path, bytes, &mut offset, true)?.ok_or_else(|| {
        DurableTransactionError::invalid(path, "transaction journal has no complete intent frame")
    })?;
    let mut journal = parse_toml_frame::<TransactionJournal>(path, intent)?;
    if !journal.transitions.is_empty() {
        return Err(DurableTransactionError::invalid(
            path,
            "immutable intent frame contains state transitions",
        ));
    }
    while let Some(frame) = decode_frame(path, bytes, &mut offset, false)? {
        let append = parse_toml_frame::<TransitionAppend>(path, frame)?;
        if append.transitions.len() != 1 {
            return Err(DurableTransactionError::invalid(
                path,
                "journal frame must contain exactly one transition",
            ));
        }
        journal.transitions.extend(append.transitions);
    }
    Ok((journal, offset))
}

pub(in crate::io::transaction) fn truncate_torn_tail(
    path: &Path,
    valid_len: usize,
) -> Result<(), DurableTransactionError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Recovery, path, source)
        })?;
    let current_len = file
        .metadata()
        .map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Recovery, path, source)
        })?
        .len();
    let valid_len = u64::try_from(valid_len).map_err(|_| {
        DurableTransactionError::operation(
            TransactionPhase::Recovery,
            path,
            io::Error::new(
                io::ErrorKind::InvalidData,
                "durable journal prefix length exceeds this platform",
            ),
        )
    })?;
    if current_len < valid_len {
        return Err(DurableTransactionError::invalid(
            path,
            "durable journal became shorter after validation",
        ));
    }
    if current_len == valid_len {
        return Ok(());
    }
    file.set_len(valid_len)
        .and_then(|()| file.sync_all())
        .map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Recovery, path, source)
        })
}
