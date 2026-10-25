use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use zircon_plugin_physics_runtime::RagdollProfile;
use zircon_runtime::core::math::{
    is_finite_vec3, try_affine_inverse, AffineInverseError, Mat4, NumericPolicy, Vec3,
};

use super::{
    prepare_ragdoll_profile_edits, RagdollBoneEdit, RagdollBoneProperty, RagdollProfileEditError,
};

/// One body's world-space drag, resolved by the host from the same admitted preview frame.
///
/// `bone_to_world` includes the bone's actual rotation and scale. It excludes the body's offset:
/// using a body/primitive matrix here would incorrectly rotate a translation twice.
#[derive(Clone, Debug, PartialEq)]
pub struct RagdollBodyTranslation {
    pub bone_path: String,
    pub bone_to_world: Mat4,
    pub world_delta: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RagdollBodyTranslationError {
    Edit(RagdollProfileEditError),
    DuplicateTarget {
        bone_path: String,
    },
    NonFiniteDelta {
        bone_path: String,
    },
    InvalidBoneFrame {
        bone_path: String,
        source: AffineInverseError,
    },
    NonFiniteOffset {
        bone_path: String,
    },
}

impl fmt::Display for RagdollBodyTranslationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(source) => fmt::Display::fmt(source, formatter),
            Self::DuplicateTarget { bone_path } => {
                write!(formatter, "duplicate body drag: {bone_path}")
            }
            Self::NonFiniteDelta { bone_path } => {
                write!(formatter, "nonfinite body drag: {bone_path}")
            }
            Self::InvalidBoneFrame { bone_path, source } => {
                write!(
                    formatter,
                    "invalid body drag bone frame {bone_path}: {source}"
                )
            }
            Self::NonFiniteOffset { bone_path } => {
                write!(formatter, "body drag offset is nonfinite: {bone_path}")
            }
        }
    }
}

impl Error for RagdollBodyTranslationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(source) => Some(source),
            Self::InvalidBoneFrame { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<RagdollProfileEditError> for RagdollBodyTranslationError {
    fn from(source: RagdollProfileEditError) -> Self {
        Self::Edit(source)
    }
}

/// Prepares a complete multi-body translation, leaving the source untouched on every failure.
///
/// No history, pose clock, runtime entity lookup or current-frame authority is created here. The
/// host must stop simulation, resolve exact paths in its current secondary session, bind all frames
/// to that same generation, then commit the returned candidate once against its document revision.
pub fn prepare_ragdoll_body_translations(
    profile: &RagdollProfile,
    translations: &[RagdollBodyTranslation],
) -> Result<Option<RagdollProfile>, RagdollBodyTranslationError> {
    profile.validate().map_err(RagdollProfileEditError::from)?;
    let bones = profile
        .bones
        .iter()
        .map(|bone| (bone.bone_path.as_str(), bone))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut edits = Vec::with_capacity(translations.len());
    for translation in translations {
        let path = translation.bone_path.as_str();
        if !seen.insert(path) {
            return Err(RagdollBodyTranslationError::DuplicateTarget {
                bone_path: path.to_owned(),
            });
        }
        let bone = bones
            .get(path)
            .ok_or_else(|| RagdollProfileEditError::MissingBone {
                bone_path: path.to_owned(),
            })?;
        if !is_finite_vec3(translation.world_delta) {
            return Err(RagdollBodyTranslationError::NonFiniteDelta {
                bone_path: path.to_owned(),
            });
        }
        let inverse = try_affine_inverse(translation.bone_to_world, NumericPolicy::STRICT)
            .map_err(|source| RagdollBodyTranslationError::InvalidBoneFrame {
                bone_path: path.to_owned(),
                source,
            })?;
        // A delta is a vector, not a point: the preview actor's translation must not enter it.
        let local_delta = inverse.transform_vector3(translation.world_delta);
        let mut offset = bone.body_offset;
        offset.translation += local_delta;
        if !is_finite_vec3(local_delta) || !is_finite_vec3(offset.translation) {
            return Err(RagdollBodyTranslationError::NonFiniteOffset {
                bone_path: path.to_owned(),
            });
        }
        edits.push(RagdollBoneEdit {
            bone_path: path.to_owned(),
            property: RagdollBoneProperty::BodyOffset(offset),
        });
    }
    prepare_ragdoll_profile_edits(profile, &edits).map_err(Into::into)
}
