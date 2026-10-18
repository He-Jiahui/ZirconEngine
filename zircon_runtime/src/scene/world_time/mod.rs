//! Per-world virtual and fixed simulation time.
//! 每个 Level 串行持有自己的时间权威；公开面提供配置、错误与可复制观测。
//! 活动固定步与外帧派生快照留在调度边界内；只读值不能替代 begin/commit/abort。

#[path = "controller.rs"]
mod controller;
#[path = "fixed_step.rs"]
mod fixed_step;
#[path = "interpolation.rs"]
mod interpolation;
#[path = "replacement.rs"]
mod replacement;
#[path = "snapshot.rs"]
mod snapshot;

// Public World-time observation and configuration surface.
pub use controller::{
    WorldTimeAdvanceError, WorldTimeControlError, WorldTimePolicyReceipt, WorldTimeState,
};
pub use fixed_step::SimulationTickId;
pub use interpolation::{FixedInterpolationContext, FixedInterpolationState};
pub use replacement::{
    WorldReplacementCheckpoint, WorldReplacementCommit, WorldReplacementError,
    WorldReplacementExpectation, WorldReplacementIdentity, WorldReplacementReason,
    WorldReplacementReceipt, WorldReplacementRejected, WorldReplacementRequest,
    WorldReplacementTimePolicy,
};

// Transaction capabilities stay inside the scene scheduler boundary.
pub(crate) use controller::WorldTimeController;
pub(crate) use fixed_step::{WorldFixedStep, WorldFixedStepError};
pub(crate) use replacement::WorldReplacementTimeDisposition;
pub(crate) use snapshot::WorldTimeSnapshot;
