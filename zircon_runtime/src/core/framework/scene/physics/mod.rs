//! Persisted scene-facing physics schema shared by assets, ECS components, and tooling.

mod ccd_mode;
mod combine_rule;
mod joint_constraint_metadata;
mod joint_constraint_serde;
mod joint_drive;
mod material_metadata;
mod skeleton_joint_binding;
mod sleep_policy;

pub use ccd_mode::PhysicsCcdMode;
pub use combine_rule::PhysicsCombineRule;
pub use joint_constraint_metadata::PhysicsJointConstraintMetadata;
pub use joint_drive::PhysicsJointDrive;
pub use material_metadata::PhysicsMaterialMetadata;
pub use skeleton_joint_binding::PhysicsSkeletonJointBinding;

mod mass_properties;
#[cfg(test)]
mod tests;
pub use mass_properties::PhysicsMassProperties;
pub use sleep_policy::PhysicsSleepPolicy;
