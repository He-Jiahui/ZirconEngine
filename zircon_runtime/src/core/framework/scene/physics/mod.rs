//! Persisted scene-facing physics schema shared by assets, ECS components, and tooling.

mod combine_rule;
mod joint_constraint_metadata;
mod joint_constraint_serde;
mod joint_drive;
mod material_metadata;
mod skeleton_joint_binding;

pub use combine_rule::PhysicsCombineRule;
pub use joint_constraint_metadata::PhysicsJointConstraintMetadata;
pub use joint_drive::PhysicsJointDrive;
pub use material_metadata::PhysicsMaterialMetadata;
pub use skeleton_joint_binding::PhysicsSkeletonJointBinding;

#[cfg(test)]
mod tests;
