use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use zircon_plugin_physics_runtime::{RagdollProfile, RagdollProfileError};
use zircon_runtime::core::framework::physics::PhysicsColliderShape;
use zircon_runtime::core::framework::scene::physics::PhysicsJointConstraintMetadata;
use zircon_runtime::core::math::{Real, Transform};

/// Properties of the profile's existing one-body-per-bone representation.
#[derive(Clone, Debug, PartialEq)]
pub enum RagdollBoneProperty {
    Shape(PhysicsColliderShape),
    Mass(Real),
    BodyOffset(Transform),
    Constraint(PhysicsJointConstraintMetadata),
    BlendWeight(Real),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RagdollBoneEdit {
    /// Exact authored path in this profile, never a display name or visible row.
    pub bone_path: String,
    pub property: RagdollBoneProperty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RagdollProfileEditError {
    InvalidProfile(RagdollProfileError),
    MissingBone { bone_path: String },
}

impl fmt::Display for RagdollProfileEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfile(error) => fmt::Display::fmt(error, formatter),
            Self::MissingBone { bone_path } => {
                write!(
                    formatter,
                    "ragdoll profile edit target does not exist: {bone_path}"
                )
            }
        }
    }
}

impl Error for RagdollProfileEditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidProfile(error) => Some(error),
            Self::MissingBone { .. } => None,
        }
    }
}

impl From<RagdollProfileError> for RagdollProfileEditError {
    fn from(error: RagdollProfileError) -> Self {
        Self::InvalidProfile(error)
    }
}

/// Prepares one complete replacement for one host-owned document transaction.
///
/// The source is immutable even when a later operation fails. Repeated edits
/// follow gesture order; no net change returns None and needs no history entry.
/// The host must bind paths to its current document/resource version and recheck
/// that version at commit. This function owns no history, version clock or scene.
pub fn prepare_ragdoll_profile_edits(
    profile: &RagdollProfile,
    edits: &[RagdollBoneEdit],
) -> Result<Option<RagdollProfile>, RagdollProfileEditError> {
    profile.validate()?;
    if edits.is_empty() {
        return Ok(None);
    }

    let indices = profile
        .bones
        .iter()
        .enumerate()
        .map(|(index, bone)| (bone.bone_path.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut candidate = profile.clone();
    for edit in edits {
        let index = indices
            .get(edit.bone_path.as_str())
            .copied()
            .ok_or_else(|| RagdollProfileEditError::MissingBone {
                bone_path: edit.bone_path.clone(),
            })?;
        let bone = &mut candidate.bones[index];
        match &edit.property {
            RagdollBoneProperty::Shape(value) => bone.shape = value.clone(),
            RagdollBoneProperty::Mass(value) => bone.mass = *value,
            RagdollBoneProperty::BodyOffset(value) => bone.body_offset = *value,
            RagdollBoneProperty::Constraint(value) => bone.constraint = value.clone(),
            RagdollBoneProperty::BlendWeight(value) => bone.blend_weight = *value,
        }
    }
    candidate.validate()?;
    if candidate.eq(profile) {
        Ok(None)
    } else {
        Ok(Some(candidate))
    }
}
