//! Bevy-inspired neutral time contracts for real, virtual, and fixed clocks.

mod clock;
mod fixed;
mod fixed_step_plan;
mod real;
mod virtual_clock;

pub use clock::Time;
pub use fixed::Fixed;
pub use fixed_step_plan::FixedStepPlan;
pub use real::Real;
pub use virtual_clock::Virtual;
