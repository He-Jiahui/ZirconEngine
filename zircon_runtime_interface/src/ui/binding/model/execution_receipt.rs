use std::fmt::Write;

use serde::{Deserialize, Serialize};

pub const UI_BINDING_TELEMETRY_ASSET_ID_MAX_BYTES: usize = 256;
pub const UI_BINDING_TELEMETRY_BINDING_ID_MAX_BYTES: usize = 128;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiBindingExecutionReceipt {
    pub asset_id: String,
    pub binding_id: String,
    pub generation: u64,
    pub execution_count: u32,
    pub miss_count: u32,
    pub error_count: u32,
    pub cost_nanos: u64,
}

impl UiBindingExecutionReceipt {
    pub fn executed(
        asset_id: &str,
        binding_id: &str,
        generation: u64,
        failed: bool,
        cost_nanos: u64,
    ) -> Self {
        Self {
            asset_id: bounded_identifier(asset_id, UI_BINDING_TELEMETRY_ASSET_ID_MAX_BYTES),
            binding_id: bounded_identifier(binding_id, UI_BINDING_TELEMETRY_BINDING_ID_MAX_BYTES),
            generation,
            execution_count: 1,
            miss_count: 0,
            error_count: u32::from(failed),
            cost_nanos,
        }
    }

    pub fn missed(asset_id: &str, binding_id: &str, generation: u64, cost_nanos: u64) -> Self {
        Self {
            asset_id: bounded_identifier(asset_id, UI_BINDING_TELEMETRY_ASSET_ID_MAX_BYTES),
            binding_id: bounded_identifier(binding_id, UI_BINDING_TELEMETRY_BINDING_ID_MAX_BYTES),
            generation,
            execution_count: 0,
            miss_count: 1,
            error_count: 0,
            cost_nanos,
        }
    }
}

fn bounded_identifier(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }

    const HASH_SUFFIX_BYTES: usize = 17;
    let mut prefix_end = max_bytes.saturating_sub(HASH_SUFFIX_BYTES);
    while prefix_end > 0 && !value.is_char_boundary(prefix_end) {
        prefix_end -= 1;
    }
    let mut bounded = String::with_capacity(max_bytes);
    bounded.push_str(&value[..prefix_end]);
    let _ = write!(bounded, "~{:016x}", stable_hash(value.as_bytes()));
    bounded
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_identifiers_are_bounded_and_keep_a_hash_suffix() {
        let asset_id = "asset/".repeat(100);
        let binding_id = "binding/".repeat(100);

        let receipt = UiBindingExecutionReceipt::executed(&asset_id, &binding_id, 7, false, 11);

        assert!(receipt.asset_id.len() <= UI_BINDING_TELEMETRY_ASSET_ID_MAX_BYTES);
        assert!(receipt.binding_id.len() <= UI_BINDING_TELEMETRY_BINDING_ID_MAX_BYTES);
        assert!(receipt.asset_id.contains('~'));
        assert!(receipt.binding_id.contains('~'));
    }
}
