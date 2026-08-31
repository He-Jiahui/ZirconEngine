//! Bevy-inspired neutral time contracts for real, virtual, and fixed clocks.

mod clock;
mod domain;
mod fixed;
mod fixed_step_plan;
mod monotonic_real;
mod policy;
mod product_policy;
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
