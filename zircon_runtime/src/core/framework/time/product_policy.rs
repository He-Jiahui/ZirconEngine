use thiserror::Error;

use super::{TimePolicy, TimePolicyError, TimePolicyTransaction};

/// Product role used to select a versioned runtime time-policy preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ProductTimeProfile {
    Client = 1,
    Headless = 2,
    Editor = 3,
    Test = 4,
}

/// Versioned product-time policy contract for runtime time and fixed-step budgeting.
///
/// `TimePolicy` owns the clock values. This type adds the profile-selected fixed-step
/// execution budget and the schema version needed to identify the configuration at a
/// BuildSet, replay, or diagnostics boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProductTimePolicy {
    version: u16,
    profile: ProductTimeProfile,
    time_policy: TimePolicy,
    max_fixed_steps_per_frame: u32,
}

impl ProductTimePolicy {
    pub const VERSION: u16 = 1;

    pub const fn new(
        version: u16,
        profile: ProductTimeProfile,
        time_policy: TimePolicy,
        max_fixed_steps_per_frame: u32,
    ) -> Self {
        Self {
            version,
            profile,
            time_policy,
            max_fixed_steps_per_frame,
        }
    }

    pub const fn version(self) -> u16 {
        self.version
    }

    pub const fn profile(self) -> ProductTimeProfile {
        self.profile
    }

    pub const fn time_policy(self) -> TimePolicy {
        self.time_policy
    }

    pub const fn max_fixed_steps_per_frame(self) -> u32 {
        self.max_fixed_steps_per_frame
    }

    pub fn validate(self) -> Result<(), ProductTimePolicyError> {
        if self.version != Self::VERSION {
            return Err(ProductTimePolicyError::UnsupportedVersion {
                requested: self.version,
            });
        }
        if self.max_fixed_steps_per_frame == 0 {
            return Err(ProductTimePolicyError::MaxFixedStepsPerFrameZero);
        }
        self.time_policy.validate()?;
        Ok(())
    }

    /// Returns the already-validated low-level transaction that clock authority accepts.
    pub fn time_policy_transaction(self) -> Result<TimePolicyTransaction, ProductTimePolicyError> {
        self.validate()?;
        Ok(TimePolicyTransaction::new(self.time_policy))
    }
}

/// Typed rejection for a product-level time-policy configuration.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProductTimePolicyError {
    #[error("unsupported product time-policy version {requested}")]
    UnsupportedVersion { requested: u16 },
    #[error("product time-policy max fixed steps per frame must be non-zero")]
    MaxFixedStepsPerFrameZero,
    #[error(transparent)]
    TimePolicy(#[from] TimePolicyError),
}

#[cfg(test)]
mod tests {
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
}
