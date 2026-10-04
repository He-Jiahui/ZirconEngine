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
