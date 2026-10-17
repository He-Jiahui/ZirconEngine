//! Bevy-inspired neutral time contracts for real, virtual, and fixed clocks.
//!
//! 提供按域区分的时钟观测、固定步计划及待验证的配置值。真实采样属于 Runtime，
//! 虚拟推进与固定步提交属于各 World；复制观测或准备配置不会取得这些推进权。

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
