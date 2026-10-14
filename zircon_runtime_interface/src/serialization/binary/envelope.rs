use serde::{Deserialize, Serialize};

use crate::serialization::PayloadHeader;

use super::value::BinaryValue;

/// Bincode-owned body following the fixed binary wire prefix.
///
/// Field order is part of wire v1. Decode reads and validates `header` before
/// deserializing `payload`; changing this order requires a wire-version bump.
///
/// `header` carries the payload's schema identity and schema version; the prefix's wire version is
/// separate. Header parsing happens in `decode_binary_header`, while target-schema and supported-
/// version checks belong to `load_binary_versioned` before that caller decodes the payload.
///
/// # 使用时机
/// The binary writer serializes this envelope after canonicalizing the payload into a bounded
/// `BinaryValue` node stream. The loader uses the same field order for current typed values and
/// older values that must enter the migration path.
#[derive(Deserialize, Serialize)]
pub(super) struct BinaryEnvelope {
    /// Domain schema identity and version, checked against the requested `VersionedSchema` by the loader.
    pub(super) header: PayloadHeader,
    /// The payload's flat binary value stream; it follows the header in the wire body.
    pub(super) payload: BinaryValue,
}
