use std::error::Error;
use std::fmt;

use zircon_plugin_physics_runtime::RagdollProfile;
use zircon_runtime::core::framework::animation::AnimationSkeletonAsset;
use zircon_runtime::core::math::{Mat4, Vec3};
use zircon_runtime::core::resource::{
    AnimationSkeletonMarker, ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceHandle,
    ResourceManager,
};

use super::{
    prepare_ragdoll_body_translations, prepare_ragdoll_profile_edits, RagdollBodyTranslation,
    RagdollBodyTranslationError, RagdollBoneEdit, RagdollBoneProperty, RagdollProfileEditError,
    RagdollProfileFromSkeleton,
};

/// An original joint slot and exact authored path in one installed skeleton payload.
/// This is process-local source identity, not a document, entity or preview input lease.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RagdollSkeletonBoneKey {
    version: ResourceActiveVersionKey,
    revision: u64,
    joint_index: u32,
    bone_path: String,
}

impl RagdollSkeletonBoneKey {
    pub fn skeleton_version(&self) -> &ResourceActiveVersionKey {
        &self.version
    }

    pub fn skeleton_revision(&self) -> u64 {
        self.revision
    }

    pub fn original_joint_slot(&self) -> u32 {
        self.joint_index
    }

    pub fn bone_path(&self) -> &str {
        &self.bone_path
    }
}

impl RagdollProfileFromSkeleton {
    /// Retained keys remain readable after reload, but checked preparation rejects stale versions.
    pub fn bone_key(&self, bone_path: &str) -> Option<RagdollSkeletonBoneKey> {
        Some(RagdollSkeletonBoneKey {
            version: self.skeleton_version().clone(),
            revision: self.skeleton_revision(),
            joint_index: self.original_joint_slot(bone_path)?,
            bone_path: bone_path.to_owned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RagdollBoundBoneEdit {
    pub bone: RagdollSkeletonBoneKey,
    pub property: RagdollBoneProperty,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RagdollBoundBodyTranslation {
    pub bone: RagdollSkeletonBoneKey,
    /// Actual admitted bone frame, including scale and excluding the body offset.
    pub bone_to_world: Mat4,
    pub world_delta: Vec3,
}

#[derive(Debug)]
pub enum RagdollBoundEditError {
    SkeletonSnapshot(ResourceActiveSnapshotError),
    SkeletonRevision { expected: u64, actual: u64 },
    WrongSkeletonBone { bone_path: String, joint_index: u32 },
    ProfileChanged,
    Edit(RagdollProfileEditError),
    Translation(RagdollBodyTranslationError),
}

impl fmt::Display for RagdollBoundEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SkeletonSnapshot(error) => {
                write!(
                    formatter,
                    "ragdoll skeleton source is unavailable: {error:?}"
                )
            }
            Self::SkeletonRevision { expected, actual } => write!(
                formatter,
                "ragdoll skeleton revision changed: expected {expected}, found {actual}"
            ),
            Self::WrongSkeletonBone {
                bone_path,
                joint_index,
            } => write!(
                formatter,
                "ragdoll target {bone_path} is not source joint slot {joint_index} in this binding"
            ),
            Self::ProfileChanged => {
                formatter.write_str("ragdoll profile changed after preparation")
            }
            Self::Edit(error) => fmt::Display::fmt(error, formatter),
            Self::Translation(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl Error for RagdollBoundEditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Translation(error) => Some(error),
            _ => None,
        }
    }
}

/// One complete validated replacement and its exact source value, without publication authority.
///
/// The existing host must bind this to its original document/source/revision/edit lease, admit
/// retained memory, then CAS and record one history operation under that authority. Save uses its
/// existing save token. Full-value equality cannot detect document ABA. Revalidation below holds
/// no resource lock after returning and is not an atomic document/resource commit or a ready lease.
#[derive(Clone, Debug)]
pub struct RagdollProfileEditCandidate {
    before: RagdollProfile,
    after: RagdollProfile,
    skeleton_version: ResourceActiveVersionKey,
    skeleton_revision: u64,
}

impl RagdollProfileEditCandidate {
    pub fn before(&self) -> &RagdollProfile {
        &self.before
    }

    pub fn after(&self) -> &RagdollProfile {
        &self.after
    }

    pub fn skeleton_version(&self) -> &ResourceActiveVersionKey {
        &self.skeleton_version
    }

    pub fn skeleton_revision(&self) -> u64 {
        self.skeleton_revision
    }

    /// Reject changed installed source or changed profile content before the host's final CAS.
    /// This deliberately does not mint a document revision, edit permit, residency or frame lease.
    pub fn validate_current(
        &self,
        resources: &ResourceManager,
        current_profile: &RagdollProfile,
    ) -> Result<(), RagdollBoundEditError> {
        check_skeleton(resources, &self.skeleton_version, self.skeleton_revision)?;
        if current_profile != &self.before {
            return Err(RagdollBoundEditError::ProfileChanged);
        }
        Ok(())
    }
}

/// Prepares one all-or-nothing batch using original joint keys and exact authored paths.
/// Repeated property edits retain gesture order. An empty or net-zero batch produces no candidate.
pub fn prepare_bound_ragdoll_profile_edits(
    resources: &ResourceManager,
    source: &RagdollProfileFromSkeleton,
    profile: &RagdollProfile,
    edits: &[RagdollBoundBoneEdit],
) -> Result<Option<RagdollProfileEditCandidate>, RagdollBoundEditError> {
    check_skeleton(
        resources,
        source.skeleton_version(),
        source.skeleton_revision(),
    )?;
    let edits = edits
        .iter()
        .map(|edit| {
            Ok(RagdollBoneEdit {
                bone_path: checked_path(source, &edit.bone)?.to_owned(),
                property: edit.property.clone(),
            })
        })
        .collect::<Result<Vec<_>, RagdollBoundEditError>>()?;
    let after =
        prepare_ragdoll_profile_edits(profile, &edits).map_err(RagdollBoundEditError::Edit)?;
    Ok(candidate(source, profile, after))
}

/// Uses the same checked source keys with the existing affine world-delta conversion.
/// Simulation, same-session frame generation and entity mapping remain host admission requirements.
pub fn prepare_bound_ragdoll_body_translations(
    resources: &ResourceManager,
    source: &RagdollProfileFromSkeleton,
    profile: &RagdollProfile,
    translations: &[RagdollBoundBodyTranslation],
) -> Result<Option<RagdollProfileEditCandidate>, RagdollBoundEditError> {
    check_skeleton(
        resources,
        source.skeleton_version(),
        source.skeleton_revision(),
    )?;
    let translations = translations
        .iter()
        .map(|translation| {
            Ok(RagdollBodyTranslation {
                bone_path: checked_path(source, &translation.bone)?.to_owned(),
                bone_to_world: translation.bone_to_world,
                world_delta: translation.world_delta,
            })
        })
        .collect::<Result<Vec<_>, RagdollBoundEditError>>()?;
    let after = prepare_ragdoll_body_translations(profile, &translations)
        .map_err(RagdollBoundEditError::Translation)?;
    Ok(candidate(source, profile, after))
}

fn check_skeleton(
    resources: &ResourceManager,
    version: &ResourceActiveVersionKey,
    revision: u64,
) -> Result<(), RagdollBoundEditError> {
    let snapshot = resources
        .snapshot_expected::<AnimationSkeletonMarker, AnimationSkeletonAsset>(
            ResourceHandle::new(version.id()),
            version,
        )
        .map_err(RagdollBoundEditError::SkeletonSnapshot)?;
    if snapshot.revision() != revision {
        return Err(RagdollBoundEditError::SkeletonRevision {
            expected: revision,
            actual: snapshot.revision(),
        });
    }
    Ok(())
}

fn checked_path<'a>(
    source: &RagdollProfileFromSkeleton,
    key: &'a RagdollSkeletonBoneKey,
) -> Result<&'a str, RagdollBoundEditError> {
    let matches = key.skeleton_version() == source.skeleton_version()
        && key.skeleton_revision() == source.skeleton_revision()
        && source
            .profile()
            .bones
            .get(key.joint_index as usize)
            .is_some_and(|bone| bone.bone_path == key.bone_path);
    if !matches {
        return Err(RagdollBoundEditError::WrongSkeletonBone {
            bone_path: key.bone_path.clone(),
            joint_index: key.joint_index,
        });
    }
    Ok(key.bone_path())
}

fn candidate(
    source: &RagdollProfileFromSkeleton,
    before: &RagdollProfile,
    after: Option<RagdollProfile>,
) -> Option<RagdollProfileEditCandidate> {
    after.map(|after| RagdollProfileEditCandidate {
        before: before.clone(),
        after,
        skeleton_version: source.skeleton_version().clone(),
        skeleton_revision: source.skeleton_revision(),
    })
}
