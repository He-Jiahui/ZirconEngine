use std::time::Duration;

use crate::core::framework::time::{ProductTimePolicy, ProductTimeProfile, TimePolicy};

/// Runtime-owned product presets for the neutral time-policy contract.
pub struct ProductTimePolicies;

impl ProductTimePolicies {
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
mod tests {
    use std::time::Duration;

    use super::{ProductTimePolicies, ProductTimePolicyDigest};
    use crate::core::framework::time::{ProductTimePolicy, ProductTimeProfile, TimePolicy};

    #[test]
    fn product_profiles_resolve_explicit_fixed_step_budgets() {
        let policies = [
            (ProductTimeProfile::Client, 8),
            (ProductTimeProfile::Headless, 16),
            (ProductTimeProfile::Editor, 4),
            (ProductTimeProfile::Test, 1),
        ];

        for (profile, expected_budget) in policies {
            let policy = ProductTimePolicies::for_profile(profile);
            assert_eq!(policy.profile(), profile);
            assert_eq!(policy.max_fixed_steps_per_frame(), expected_budget);
            assert_eq!(
                policy.time_policy().fixed_timestep(),
                Duration::from_micros(15_625)
            );
            assert_eq!(policy.time_policy().virtual_relative_speed(), 1.0);
            policy.validate().expect("built-in policy must validate");
        }
    }

    #[test]
    fn product_policy_digest_is_stable_and_canonicalizes_negative_zero() {
        let client = ProductTimePolicies::for_profile(ProductTimeProfile::Client);
        let headless = ProductTimePolicies::for_profile(ProductTimeProfile::Headless);
        assert_eq!(
            ProductTimePolicyDigest::from_policy(client),
            ProductTimePolicyDigest::from_policy(client)
        );
        assert_ne!(
            ProductTimePolicyDigest::from_policy(client),
            ProductTimePolicyDigest::from_policy(headless)
        );

        let policy = |speed| {
            ProductTimePolicy::new(
                ProductTimePolicy::VERSION,
                ProductTimeProfile::Client,
                TimePolicy::new(
                    Duration::from_millis(250),
                    speed,
                    Duration::from_micros(15_625),
                ),
                8,
            )
        };
        assert_eq!(
            ProductTimePolicyDigest::from_policy(policy(0.0)),
            ProductTimePolicyDigest::from_policy(policy(-0.0))
        );
    }
}
