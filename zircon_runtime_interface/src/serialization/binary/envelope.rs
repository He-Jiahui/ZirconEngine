use serde::{Deserialize, Serialize};

use crate::serialization::PayloadHeader;

use super::value::BinaryValue;

/// Bincode-owned body following the fixed binary wire prefix.
///
/// Field order is part of wire v1. Decode reads and validates `header` before
/// deserializing `payload`; changing this order requires a wire-version bump.
#[derive(Deserialize, Serialize)]
pub(super) struct BinaryEnvelope {
    pub(super) header: PayloadHeader,
    pub(super) payload: BinaryValue,
}
