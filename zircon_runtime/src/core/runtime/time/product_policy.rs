use std::time::Duration;

use crate::core::framework::time::{ProductTimePolicy, ProductTimeProfile, TimePolicy};

/// Runtime-owned product presets for the neutral time-policy contract.
/// 产品角色到默认时间配置的无状态选择器；实际采纳由会话或宿主完成。
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
/// 保存策略值的 BLAKE3 摘要，不借用原策略，也不构成策略已验证或已应用的证明。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProductTimePolicyDigest([u8; blake3::OUT_LEN]);

impl ProductTimePolicyDigest {
    /// Hashes every policy field in a fixed little-endian representation.
    /// 对 `policy` 的版本、角色、时钟值及预算按固定字段顺序编码并散列，不执行值域校验。
    /// 时长分别编码秒与纳秒；倍率只统一正负零，其余位模式保留，不能据摘要推断配置有效。
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

    /// 借用本摘要的固定长度字节；借用期受 `self` 限制，不返回策略或活动 Runtime 的引用。
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
