mod descriptor;
mod params;
#[cfg(feature = "backend-jolt")]
mod projection;

pub use descriptor::ConstraintDesc;
pub use params::{AxisConstraint, JointParams, JointSpring};
#[cfg(feature = "backend-jolt")]
pub(crate) use projection::{project_constraint, ProjectedBodies};
