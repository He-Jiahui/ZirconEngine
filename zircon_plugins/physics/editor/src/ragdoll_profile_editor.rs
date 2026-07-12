use zircon_plugin_physics_runtime::{RagdollBoneProfile, RagdollProfile, RagdollProfileError};
use zircon_runtime::core::framework::physics::PhysicsColliderShape;
use zircon_runtime::core::framework::scene::physics::PhysicsJointConstraintMetadata;
use zircon_runtime::core::math::{Real, Transform};

const DEFAULT_BONE_MASS: Real = 1.0;
const DEFAULT_ROOT_RADIUS: Real = 0.15;
const DEFAULT_ROOT_HALF_HEIGHT: Real = 0.2;
const MIN_BONE_RADIUS: Real = 0.05;
const MAX_BONE_RADIUS: Real = 0.25;
const MIN_BONE_HALF_HEIGHT: Real = 0.05;

#[derive(Clone, Debug, PartialEq)]
pub struct RagdollSkeletonBone {
    pub bone_path: String,
    pub parent_bone_path: Option<String>,
    pub local_transform: Transform,
}

impl RagdollSkeletonBone {
    pub fn new(
        bone_path: impl Into<String>,
        parent_bone_path: Option<&str>,
        local_transform: Transform,
    ) -> Self {
        Self {
            bone_path: bone_path.into(),
            parent_bone_path: parent_bone_path.map(str::to_string),
            local_transform,
        }
    }
}

pub fn generate_initial_ragdoll_profile(
    profile_id: impl Into<String>,
    skeleton: &[RagdollSkeletonBone],
) -> Result<RagdollProfile, RagdollProfileError> {
    let bones = skeleton
        .iter()
        .map(|bone| {
            let length = bone.local_transform.translation.length();
            let shape = if bone.parent_bone_path.is_some() {
                PhysicsColliderShape::Capsule {
                    radius: (length * 0.2).clamp(MIN_BONE_RADIUS, MAX_BONE_RADIUS),
                    half_height: (length * 0.5).max(MIN_BONE_HALF_HEIGHT),
                }
            } else {
                PhysicsColliderShape::Capsule {
                    radius: DEFAULT_ROOT_RADIUS,
                    half_height: DEFAULT_ROOT_HALF_HEIGHT,
                }
            };
            RagdollBoneProfile {
                bone_path: bone.bone_path.clone(),
                parent_bone_path: bone.parent_bone_path.clone(),
                shape,
                mass: DEFAULT_BONE_MASS,
                body_offset: Transform::default(),
                constraint: PhysicsJointConstraintMetadata::default(),
                blend_weight: 1.0,
            }
        })
        .collect();
    let profile = RagdollProfile {
        id: profile_id.into(),
        bones,
    };
    profile.validate()?;
    Ok(profile)
}
