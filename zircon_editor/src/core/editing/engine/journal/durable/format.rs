use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    DurableJournalError, JournalDocumentKey, JOURNAL_FORMAT_VERSION, JOURNAL_MAGIC,
    MAX_HEADER_BYTES,
};

#[derive(Serialize, Deserialize)]
pub(super) struct JournalHeader {
    pub(super) format_version: u16,
    pub(super) document_key: String,
    pub(super) source_path: PathBuf,
    pub(super) base_sequence: u64,
}

pub(super) fn write_header(
    file: &mut File,
    document: &JournalDocumentKey,
    base_sequence: u64,
    path: &Path,
) -> Result<(), DurableJournalError> {
    let header = serde_json::to_vec(&JournalHeader {
        format_version: JOURNAL_FORMAT_VERSION,
        document_key: document.as_str().to_string(),
        source_path: document.source_path().to_path_buf(),
        base_sequence,
    })
    .map_err(|source| DurableJournalError::HeaderEncode {
        path: path.to_path_buf(),
        source,
    })?;
    if header.len() > MAX_HEADER_BYTES {
        return Err(DurableJournalError::InvalidHeader {
            path: path.to_path_buf(),
        });
    }
    file.write_all(&JOURNAL_MAGIC)
        .and_then(|()| file.write_all(&(header.len() as u32).to_le_bytes()))
        .and_then(|()| file.write_all(blake3::hash(&header).as_bytes()))
        .and_then(|()| file.write_all(&header))
        .and_then(|()| file.sync_data())
        .map_err(|source| DurableJournalError::Io {
            operation: "write durable journal header",
            path: path.to_path_buf(),
            source,
        })
}

pub(super) fn read_header(
    bytes: &[u8],
    cursor: &mut usize,
    path: &Path,
) -> Result<JournalHeader, DurableJournalError> {
    if take(bytes, cursor, JOURNAL_MAGIC.len()) != Some(JOURNAL_MAGIC.as_slice()) {
        return Err(DurableJournalError::UnsupportedFormat {
            path: path.to_path_buf(),
        });
    }
    let Some(length_bytes) = take(bytes, cursor, 4) else {
        return Err(DurableJournalError::InvalidHeader {
            path: path.to_path_buf(),
        });
    };
    let Some(length) = little_endian_u32(length_bytes) else {
        return Err(DurableJournalError::InvalidHeader {
            path: path.to_path_buf(),
        });
    };
    let length = length as usize;
    if length > MAX_HEADER_BYTES {
        return Err(DurableJournalError::InvalidHeader {
            path: path.to_path_buf(),
        });
    }
    let Some(expected_digest) = take(bytes, cursor, blake3::OUT_LEN) else {
        return Err(DurableJournalError::InvalidHeader {
            path: path.to_path_buf(),
        });
    };
    let Some(payload) = take(bytes, cursor, length) else {
        return Err(DurableJournalError::InvalidHeader {
            path: path.to_path_buf(),
        });
    };
    if blake3::hash(payload).as_bytes() != expected_digest {
        return Err(DurableJournalError::InvalidHeader {
            path: path.to_path_buf(),
        });
    }
    let header: JournalHeader =
        serde_json::from_slice(payload).map_err(|_| DurableJournalError::InvalidHeader {
            path: path.to_path_buf(),
        })?;
    if header.format_version != JOURNAL_FORMAT_VERSION {
        return Err(DurableJournalError::UnsupportedFormat {
            path: path.to_path_buf(),
        });
    }
    Ok(header)
}

pub(super) fn take<'a>(bytes: &'a [u8], cursor: &mut usize, length: usize) -> Option<&'a [u8]> {
    let end = cursor.checked_add(length)?;
    let slice = bytes.get(*cursor..end)?;
    *cursor = end;
    Some(slice)
}

pub(super) fn little_endian_u32(bytes: &[u8]) -> Option<u32> {
    let bytes: [u8; 4] = bytes.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

pub(super) fn little_endian_u64(bytes: &[u8]) -> Option<u64> {
    let bytes: [u8; 8] = bytes.try_into().ok()?;
    Some(u64::from_le_bytes(bytes))
}
