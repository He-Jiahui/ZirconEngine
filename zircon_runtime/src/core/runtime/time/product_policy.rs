use std::time::Duration;

use crate::core::framework::time::{ProductTimePolicy, ProductTimeProfile, TimePolicy};

/// Runtime-owned product presets for the neutral time-policy contract.
pub struct ProductTimePolicies;

impl ProductTimePolicies {
    /// 选择客户端、无头、编辑器或测试宿主的默认帧预算和 World 时间策略。
    /// 返回值由宿主在会话启动时采纳，不会追溯修改已创建的 Level。
    pub fn for_profile(profile: ProductTimeProfile) -> ProductTimePolicy {
        let (max_fixed_steps_per_frame, virtual_max_delta) = match profile {
            ProductTimeProfile::Client => (8, Duration::from_millis(250)),
            ProductTimeProfile::Headless => (16, Duration::from_millis(250)),
            ProductTimeProfile::Editor => (4, Duration::from_millis(250)),
            ProductTimeProfile::Test => (1, Duration::from_millis(250)),
        };
        ProductTimePolicy::new(
            ProductTimePolicy::VERSION,
            profile,
            TimePolicy::new(virtual_max_delta, 1.0, Duration::from_micros(15_625)),
            max_fixed_steps_per_frame,
        )
    }
}

/// Canonical BLAKE3 digest for a versioned product time-policy value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProductTimePolicyDigest([u8; blake3::OUT_LEN]);

impl ProductTimePolicyDigest {
    /// Hashes every policy field in a fixed little-endian representation.
    pub fn from_policy(policy: ProductTimePolicy) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"zircon.product-time-policy");
        hasher.update(&policy.version().to_le_bytes());
        hasher.update(&[policy.profile() as u8]);
        update_duration(&mut hasher, policy.time_policy().virtual_max_delta());
        let relative_speed_bits = canonical_f64_bits(policy.time_policy().virtual_relative_speed());
        hasher.update(&relative_speed_bits.to_le_bytes());
        update_duration(&mut hasher, policy.time_policy().fixed_timestep());
        hasher.update(&policy.max_fixed_steps_per_frame().to_le_bytes());
        Self(*hasher.finalize().as_bytes())
    }

    pub const fn as_bytes(&self) -> &[u8; blake3::OUT_LEN] {
        &self.0
    }
}

fn update_duration(hasher: &mut blake3::Hasher, duration: Duration) {
    hasher.update(&duration.as_secs().to_le_bytes());
    hasher.update(&duration.subsec_nanos().to_le_bytes());
}

fn canonical_f64_bits(value: f64) -> u64 {
    if value == 0.0 {
        0.0_f64.to_bits()
    } else {
        value.to_bits()
    }
}

#[cfg(test)]
#[path = "tests/product_policy.rs"]
mod tests;
