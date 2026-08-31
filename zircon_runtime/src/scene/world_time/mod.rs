//! Per-world virtual and fixed simulation time.

mod controller;
mod fixed_step;
mod interpolation;
mod snapshot;

// Public World-time observation and configuration surface.
pub use controller::{
    WorldTimeAdvanceError, WorldTimeControlError, WorldTimePolicyReceipt, WorldTimeState,
};
pub use fixed_step::SimulationTickId;
pub use interpolation::{FixedInterpolationContext, FixedInterpolationState};

// Transaction capabilities stay inside the scene scheduler boundary.
pub(crate) use controller::WorldTimeController;
pub(crate) use fixed_step::{WorldFixedStep, WorldFixedStepError};
pub(crate) use snapshot::WorldTimeSnapshot;
