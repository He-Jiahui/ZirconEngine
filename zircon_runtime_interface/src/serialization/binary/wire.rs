use bincode::Options;

use crate::serialization::LoadError;

pub(super) const BINARY_MAGIC: [u8; 8] = *b"ZRPAYLD\0";
pub(super) const BINARY_WIRE_VERSION: u16 = 1;
pub(super) const BINARY_PREFIX_LEN: usize = BINARY_MAGIC.len() + size_of::<u16>();
pub(in crate::serialization) const MAX_BINARY_BODY_BYTES: usize = 64 * 1024 * 1024;
pub(in crate::serialization) const MAX_BINARY_CONTAINER_ENTRIES: usize = 1_000_000;
pub(in crate::serialization) const MAX_BINARY_DEPTH: usize = 128;
pub(in crate::serialization) const MAX_BINARY_NODES: usize = 2_000_000;
pub(in crate::serialization) const MAX_BINARY_STRING_BYTES: usize = 16 * 1024 * 1024;

/// Pins every bincode compatibility option instead of inheriting crate defaults.
pub(super) fn options() -> impl Options {
    bincode::DefaultOptions::new()
        .with_varint_encoding()
        .with_little_endian()
        .reject_trailing_bytes()
}

pub(super) fn append_prefix(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&BINARY_MAGIC);
    bytes.extend_from_slice(&BINARY_WIRE_VERSION.to_le_bytes());
}

pub(super) fn body_after_valid_prefix(bytes: &[u8]) -> Result<&[u8], LoadError> {
    if bytes.len() < BINARY_PREFIX_LEN {
        return Err(LoadError::BinaryHeaderTruncated {
            expected: BINARY_PREFIX_LEN,
            found: bytes.len(),
        });
    }

    let mut found_magic = [0; BINARY_MAGIC.len()];
    found_magic.copy_from_slice(&bytes[..BINARY_MAGIC.len()]);
    if found_magic != BINARY_MAGIC {
        return Err(LoadError::BinaryMagicMismatch { found: found_magic });
    }

    let version_offset = BINARY_MAGIC.len();
    let found_version = u16::from_le_bytes([bytes[version_offset], bytes[version_offset + 1]]);
    if found_version != BINARY_WIRE_VERSION {
        return Err(LoadError::UnsupportedBinaryWireVersion {
            found: found_version,
            supported: BINARY_WIRE_VERSION,
        });
    }

    let body = &bytes[BINARY_PREFIX_LEN..];
    if body.len() > MAX_BINARY_BODY_BYTES {
        return Err(LoadError::BinaryPayloadTooLarge {
            max: MAX_BINARY_BODY_BYTES,
            found: body.len(),
        });
    }

    Ok(body)
}
