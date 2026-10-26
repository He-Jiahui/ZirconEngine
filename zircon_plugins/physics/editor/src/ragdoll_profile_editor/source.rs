use std::error::Error;
use std::fmt;

use zircon_plugin_physics_runtime::{RagdollProfile, RagdollProfileError};
use zircon_runtime::core::framework::animation::AnimationSkeletonAsset;
use zircon_runtime::core::math::{Quat, Transform, Vec3};
use zircon_runtime::core::resource::{
    AnimationSkeletonMarker, ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceHandle,
    ResourceManager,
};

use super::{generate_initial_ragdoll_profile, RagdollSkeletonBone};

/// A generation candidate tied to one installed skeleton, without document or scene authority.
#[derive(Clone, Debug)]
pub struct RagdollProfileFromSkeleton {
    profile: RagdollProfile,
    skeleton_version: ResourceActiveVersionKey,
    skeleton_revision: u64,
}

impl RagdollProfileFromSkeleton {
    pub fn profile(&self) -> &RagdollProfile {
        &self.profile
    }

    pub fn skeleton_version(&self) -> &ResourceActiveVersionKey {
        &self.skeleton_version
    }

    pub fn skeleton_revision(&self) -> u64 {
        self.skeleton_revision
    }

    /// Resolves an exact authored profile path to the original skeleton payload slot.
    /// This is not a runtime entity, filtered row, or a current picking authorization.
    pub fn original_joint_slot(&self, bone_path: &str) -> Option<u32> {
        self.profile
            .bones
            .iter()
            .position(|bone| bone.bone_path == bone_path)
            .and_then(|index| u32::try_from(index).ok())
    }
}

#[derive(Debug)]
pub enum RagdollProfileFromSkeletonError {
    Snapshot(ResourceActiveSnapshotError),
    TooManyBones {
        bone_count: usize,
    },
    PathCountMismatch {
        bone_count: usize,
        path_count: usize,
    },
    ParentOutOfBounds {
        joint_index: usize,
        parent_index: u32,
    },
    NonFiniteTransform {
        joint_index: usize,
    },
    Profile(RagdollProfileError),
}

impl fmt::Display for RagdollProfileFromSkeletonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => {
                write!(formatter, "installed skeleton cannot be read: {error:?}")
            }
            Self::TooManyBones { bone_count } => {
                write!(formatter, "skeleton joint slots exceed u32: {bone_count}")
            }
            Self::PathCountMismatch {
                bone_count,
                path_count,
            } => write!(
                formatter,
                "skeleton has {bone_count} joints but {path_count} authored profile paths"
            ),
            Self::ParentOutOfBounds {
                joint_index,
                parent_index,
            } => write!(
                formatter,
                "skeleton joint {joint_index} has an out-of-bounds parent {parent_index}"
            ),
            Self::NonFiniteTransform { joint_index } => {
                write!(
                    formatter,
                    "skeleton joint {joint_index} has nonfinite source TRS"
                )
            }
            Self::Profile(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl Error for RagdollProfileFromSkeletonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            _ => None,
        }
    }
}

/// Uses the existing generator with one atomically paired, version-checked resource snapshot.
///
/// `source_paths` supplies one exact authored path per original joint slot. Names and visible rows
/// are not identities; no paths are inferred, trimmed or normalized. Source local TRS is checked
/// for finiteness and passed unchanged to the generator's existing capsule-length heuristic.
/// This neither validates an animation pose nor changes the generator's identity body offsets.
///
/// The host must recheck current resource/document identity and readiness before commit, then own
/// its history/save transaction. A retained candidate proves no current readiness, residency,
/// entity binding, picking, preview frame or command completion.
pub fn generate_ragdoll_profile_from_skeleton(
    resources: &ResourceManager,
    expected: &ResourceActiveVersionKey,
    profile_id: impl Into<String>,
    source_paths: &[String],
) -> Result<RagdollProfileFromSkeleton, RagdollProfileFromSkeletonError> {
    use RagdollProfileFromSkeletonError as SourceError;

    let snapshot = resources
        .snapshot_expected::<AnimationSkeletonMarker, AnimationSkeletonAsset>(
            ResourceHandle::new(expected.id()),
            expected,
        )
        .map_err(SourceError::Snapshot)?;
    let bone_count = snapshot.bones.len();
    u32::try_from(bone_count).map_err(|_| SourceError::TooManyBones { bone_count })?;
    if source_paths.len() != bone_count {
        return Err(SourceError::PathCountMismatch {
            bone_count,
            path_count: source_paths.len(),
        });
    }

    let mut skeleton = Vec::with_capacity(bone_count);
    for (joint_index, bone) in snapshot.bones.iter().enumerate() {
        let parent_path = match bone.parent_index {
            Some(parent_index) => Some(
                usize::try_from(parent_index)
                    .ok()
                    .and_then(|index| source_paths.get(index))
                    .ok_or(SourceError::ParentOutOfBounds {
                        joint_index,
                        parent_index,
                    })?
                    .as_str(),
            ),
            None => None,
        };
        let local_transform = Transform {
            translation: Vec3::from_array(bone.local_translation),
            rotation: Quat::from_array(bone.local_rotation),
            scale: Vec3::from_array(bone.local_scale),
        };
        if !local_transform.translation.is_finite()
            || !local_transform.rotation.is_finite()
            || !local_transform.scale.is_finite()
        {
            return Err(SourceError::NonFiniteTransform { joint_index });
        }
        skeleton.push(RagdollSkeletonBone::new(
            &source_paths[joint_index],
            parent_path,
            local_transform,
        ));
    }
    let profile =
        generate_initial_ragdoll_profile(profile_id, &skeleton).map_err(SourceError::Profile)?;
    Ok(RagdollProfileFromSkeleton {
        profile,
        skeleton_version: expected.clone(),
        skeleton_revision: snapshot.revision(),
    })
}
