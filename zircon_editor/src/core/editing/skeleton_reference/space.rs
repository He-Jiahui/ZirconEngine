use zircon_runtime::core::framework::animation::AnimationSkeletonBoneAsset;
use zircon_runtime::core::math::{
    is_finite_mat4, is_finite_vec3, Mat4, NumericError, NumericPolicy, NumericValue, Quat,
    Transform, UnitQuaternion, Vec3,
};

use super::hierarchy::SkeletonReferenceHierarchy;

/// Spaces derived solely from one immutable skeleton reference payload.
///
/// SkeletonRoot excludes the preview mesh/actor placement, root motion and animation evaluation.
/// A caller needing world or UE Mesh Relative values must obtain them from the actual preview host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SkeletonReferenceSpace {
    Local,
    SkeletonRoot,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub(crate) enum SkeletonReferenceTransformError {
    #[error("reference bone source slot {joint_index} has invalid local data: {source}")]
    InvalidLocal {
        joint_index: usize,
        #[source]
        source: NumericError,
    },
    #[error("reference matrix is non-finite after composing source slot {joint_index}")]
    NonFiniteMatrix { joint_index: usize },
}

/// The caller supplies the already-validated hierarchy and a slot from the same bone slice.
pub(super) fn reference_matrix(
    bones: &[AnimationSkeletonBoneAsset],
    hierarchy: &SkeletonReferenceHierarchy,
    index: usize,
    space: SkeletonReferenceSpace,
) -> Result<Mat4, SkeletonReferenceTransformError> {
    let mut chain = Vec::new();
    let mut current = Some(index);
    while let Some(index) = current {
        chain.push(index);
        if space == SkeletonReferenceSpace::Local {
            break;
        }
        current = hierarchy.parents[index];
    }

    // Parent slots need not precede children. Preserve the source ordering and avoid recursive
    // traversal, matrix inversion and TRS decomposition (which would discard inherited shear).
    let mut result = Mat4::IDENTITY;
    for index in chain.into_iter().rev() {
        let local = local_transform(&bones[index]).map_err(|source| {
            SkeletonReferenceTransformError::InvalidLocal {
                joint_index: index,
                source,
            }
        })?;
        result *= local.matrix();
        if !is_finite_mat4(result) {
            return Err(SkeletonReferenceTransformError::NonFiniteMatrix { joint_index: index });
        }
    }
    Ok(result)
}

fn local_transform(bone: &AnimationSkeletonBoneAsset) -> Result<Transform, NumericError> {
    let translation = Vec3::from_array(bone.local_translation);
    if !is_finite_vec3(translation) {
        return Err(NumericError::NonFinite {
            value: NumericValue::Translation,
        });
    }
    let scale = Vec3::from_array(bone.local_scale);
    if !is_finite_vec3(scale) {
        return Err(NumericError::NonFinite {
            value: NumericValue::Scale,
        });
    }
    let rotation =
        UnitQuaternion::try_new(Quat::from_array(bone.local_rotation), NumericPolicy::STRICT)?
            .into_quat();
    // Displaying a reference matrix does not require an inverse: finite zero/negative scale is
    // retained. The borrowed raw reference values and the resource payload are never rewritten.
    Ok(Transform {
        translation,
        rotation,
        scale,
    })
}
