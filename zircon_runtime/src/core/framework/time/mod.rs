//! Bevy-inspired neutral time contracts for real, virtual, and fixed clocks.

#[path = "clock.rs"]
mod clock;
#[path = "domain/mod.rs"]
mod domain;
#[path = "fixed.rs"]
mod fixed;
#[path = "fixed_step_plan.rs"]
mod fixed_step_plan;
#[path = "monotonic_real.rs"]
mod monotonic_real;
#[path = "policy.rs"]
mod policy;
#[path = "product_policy.rs"]
mod product_policy;
#[path = "virtual_clock.rs"]
mod virtual_clock;

pub use clock::Time;
pub use domain::{
    ClockDomainDescriptor, ClockDomainId, ClockDomainMarker, ClockDomainRegistry, ClockDomainStamp,
    ClockDomainUnit,
};
pub use fixed::Fixed;
pub use fixed_step_plan::FixedStepPlan;
pub use monotonic_real::MonotonicReal;
pub use policy::{TimePolicy, TimePolicyError, TimePolicyTransaction};
pub use product_policy::{ProductTimePolicy, ProductTimePolicyError, ProductTimeProfile};
pub use virtual_clock::Virtual;
