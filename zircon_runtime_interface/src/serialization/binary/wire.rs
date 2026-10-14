//! Binary wire v1's fixed prefix and body limits.
//!
//! The ten-byte prefix identifies this format and its wire revision; it is followed by a bounded
//! bincode body containing [`super::envelope::BinaryEnvelope`]. The body limit excludes the prefix.
//! Schema identity/version and node-domain limits are handled at their respective header and
//! `BinaryValue` validation layers.

use crate::serialization::{LoadError, PositionalEncoding};

/// Eight-byte discriminator for this serialization format.
pub(super) const BINARY_MAGIC: [u8; 8] = *b"ZRPAYLD\0";
/// Revision of the prefix/body encoding, independent of the payload schema version.
pub(super) const BINARY_WIRE_VERSION: u16 = 1;
/// Fixed prefix width: magic followed by the little-endian wire revision.
pub(super) const BINARY_PREFIX_LEN: usize = BINARY_MAGIC.len() + size_of::<u16>();
/// Maximum encoded bincode body bytes; the fixed prefix is not included.
pub(in crate::serialization) const MAX_BINARY_BODY_BYTES: usize = 64 * 1024 * 1024;
/// Maximum entries in any array or object in the decoded `BinaryValue` domain.
pub(in crate::serialization) const MAX_BINARY_CONTAINER_ENTRIES: usize = 1_000_000;
/// Maximum nested array/object depth accepted by binary value conversion.
pub(in crate::serialization) const MAX_BINARY_DEPTH: usize = 128;
/// Maximum flat nodes accepted in a binary value stream.
pub(in crate::serialization) const MAX_BINARY_NODES: usize = 2_000_000;
/// Maximum UTF-8 byte length of a string or object key in the binary value domain.
pub(in crate::serialization) const MAX_BINARY_STRING_BYTES: usize = 16 * 1024 * 1024;

/// Pins every bincode compatibility option instead of inheriting crate defaults.
/// 变长整数、小端序和尾随字节策略均属于二进制 v1 的持久化契约。
pub(super) fn options() -> PositionalEncoding {
    PositionalEncoding::VariableLittleEndian
}

/// Appends the fixed v1 prefix; the caller appends the separately bounded body afterward.
pub(super) fn append_prefix(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&BINARY_MAGIC);
    bytes.extend_from_slice(&BINARY_WIRE_VERSION.to_le_bytes());
}

/// Validates the fixed prefix and body length, then lends the body slice to the decoder.
///
/// # 参数
/// - `bytes`：完整输入，包含前缀及其后的 bincode body。
///
/// # 返回值
/// 不含前缀的 body 子切片；其生命周期受输入切片约束。
///
/// # 错误
/// 前缀不完整、magic 不匹配、wire version 不受支持或 body 超过
/// [`MAX_BINARY_BODY_BYTES`] 时返回对应 [`LoadError`]。此处不读取封套或 payload。
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

    // Prefix 已确认完整且版本受支持后，单独限制 body；固定前缀不消耗 body 预算。
    let body = &bytes[BINARY_PREFIX_LEN..];
    if body.len() > MAX_BINARY_BODY_BYTES {
        return Err(LoadError::BinaryPayloadTooLarge {
            max: MAX_BINARY_BODY_BYTES,
            found: body.len(),
        });
    }

    Ok(body)
}
