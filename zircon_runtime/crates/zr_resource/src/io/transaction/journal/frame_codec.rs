//! Bounded checksummed WAL frame encoding and decoding.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::super::error::{DurableTransactionError, TransactionPhase};
use super::super::schema::JournalTransition;

pub(super) const FRAME_HEADER_BYTES: usize = 8 + blake3::OUT_LEN;
pub(in crate::io::transaction) const MAX_JOURNAL_BYTES: usize = 128 * 1024 * 1024;
const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn transition_frame(
    path: &Path,
    transition: JournalTransition,
    phase: TransactionPhase,
) -> Result<Vec<u8>, DurableTransactionError> {
    let bytes = toml::to_string_pretty(&TransitionAppend {
        transitions: vec![transition],
    })
    .map_err(|error| {
        DurableTransactionError::operation(
            phase,
            path,
            io::Error::new(io::ErrorKind::InvalidData, error.to_string()),
        )
    })?;
    encode_frame(bytes.as_bytes())
}

pub(in crate::io::transaction) fn encode_frame(
    payload: &[u8],
) -> Result<Vec<u8>, DurableTransactionError> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err(DurableTransactionError::operation(
            TransactionPhase::Stage,
            PathBuf::from("<journal-frame>"),
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "durable transaction journal frame exceeds its bounded size",
            ),
        ));
    }
    let mut frame = Vec::with_capacity(FRAME_HEADER_BYTES + payload.len());
    frame.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    frame.extend_from_slice(blake3::hash(payload).as_bytes());
    frame.extend_from_slice(payload);
    Ok(frame)
}

pub(super) fn decode_frame<'a>(
    path: &Path,
    bytes: &'a [u8],
    offset: &mut usize,
    intent: bool,
) -> Result<Option<&'a [u8]>, DurableTransactionError> {
    let remaining = &bytes[*offset..];
    if remaining.is_empty() {
        return Ok(None);
    }
    if remaining.len() < FRAME_HEADER_BYTES {
        return if intent {
            Err(DurableTransactionError::invalid(
                path,
                "immutable intent frame header is incomplete",
            ))
        } else {
            Ok(None)
        };
    }
    let mut length_bytes = [0_u8; 8];
    length_bytes.copy_from_slice(&remaining[..8]);
    let length = u64::from_le_bytes(length_bytes);
    let length = usize::try_from(length).map_err(|_| {
        DurableTransactionError::invalid(path, "journal frame length exceeds this platform")
    })?;
    if length > MAX_FRAME_BYTES {
        return Err(DurableTransactionError::invalid(
            path,
            "journal frame exceeds its bounded size",
        ));
    }
    let end = FRAME_HEADER_BYTES
        .checked_add(length)
        .ok_or_else(|| DurableTransactionError::invalid(path, "journal frame length overflows"))?;
    if remaining.len() < end {
        return if intent {
            Err(DurableTransactionError::invalid(
                path,
                "immutable intent frame payload is incomplete",
            ))
        } else {
            Ok(None)
        };
    }
    let payload = &remaining[FRAME_HEADER_BYTES..end];
    let expected = &remaining[8..FRAME_HEADER_BYTES];
    if blake3::hash(payload).as_bytes() != expected {
        if !intent && remaining.len() == end {
            return Ok(None);
        }
        return Err(DurableTransactionError::invalid(
            path,
            "journal frame checksum is invalid",
        ));
    }
    *offset += end;
    Ok(Some(payload))
}

pub(super) fn parse_toml_frame<T: for<'de> Deserialize<'de>>(
    path: &Path,
    bytes: &[u8],
) -> Result<T, DurableTransactionError> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        DurableTransactionError::invalid(path, format!("journal frame is not UTF-8: {error}"))
    })?;
    toml::from_str(source).map_err(|source| DurableTransactionError::JournalDeserialize {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Deserialize, Serialize)]
pub(super) struct TransitionAppend {
    pub(super) transitions: Vec<JournalTransition>,
}
