//! Persisted scene-facing physics schema shared by assets, ECS components, and tooling.

#[path = "ccd_mode.rs"]
mod ccd_mode;
#[path = "combine_rule.rs"]
mod combine_rule;
#[path = "joint_constraint_metadata.rs"]
mod joint_constraint_metadata;
#[path = "joint_constraint_serde.rs"]
mod joint_constraint_serde;
#[path = "joint_drive.rs"]
mod joint_drive;
#[path = "material_metadata.rs"]
mod material_metadata;
#[path = "skeleton_joint_binding.rs"]
mod skeleton_joint_binding;
#[path = "sleep_policy.rs"]
mod sleep_policy;

pub use ccd_mode::PhysicsCcdMode;
pub use combine_rule::PhysicsCombineRule;
pub use joint_constraint_metadata::PhysicsJointConstraintMetadata;
pub use joint_drive::PhysicsJointDrive;
pub use material_metadata::PhysicsMaterialMetadata;
pub use skeleton_joint_binding::PhysicsSkeletonJointBinding;

#[path = "mass_properties.rs"]
mod mass_properties;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
pub use mass_properties::PhysicsMassProperties;
pub use sleep_policy::PhysicsSleepPolicy;
