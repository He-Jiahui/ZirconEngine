use std::time::Duration;

use super::{ProductTimePolicy, ProductTimePolicyError, ProductTimeProfile};
use crate::core::framework::time::{TimePolicy, TimePolicyError};

#[test]
fn contract_rejects_unsupported_versions_and_zero_budgets() {
    let unsupported = ProductTimePolicy::new(
        ProductTimePolicy::VERSION + 1,
        ProductTimeProfile::Client,
        TimePolicy::default(),
        8,
    );
    assert_eq!(
        unsupported.validate(),
        Err(ProductTimePolicyError::UnsupportedVersion {
            requested: ProductTimePolicy::VERSION + 1,
        })
    );

    let zero_budget = ProductTimePolicy::new(
        ProductTimePolicy::VERSION,
        ProductTimeProfile::Client,
        TimePolicy::default(),
        0,
    );
    assert_eq!(
        zero_budget.validate(),
        Err(ProductTimePolicyError::MaxFixedStepsPerFrameZero)
    );
}

#[test]
fn contract_propagates_neutral_time_policy_rejections() {
    let invalid = ProductTimePolicy::new(
        ProductTimePolicy::VERSION,
        ProductTimeProfile::Client,
        TimePolicy::new(Duration::ZERO, 1.0, Duration::from_millis(16)),
        8,
    );

    assert_eq!(
        invalid.validate(),
        Err(ProductTimePolicyError::TimePolicy(
            TimePolicyError::VirtualMaxDeltaZero
        ))
    );
}

#[test]
fn headless_profile_preserves_its_stable_discriminant() {
    assert_eq!(ProductTimeProfile::Headless as u8, 2);
}
