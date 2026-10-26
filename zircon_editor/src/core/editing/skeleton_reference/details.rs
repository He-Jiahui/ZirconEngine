use zircon_runtime::core::math::Real;

use super::SkeletonBoneKey;

/// A parent in the same installed skeleton, independent of tree filtering and row order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SkeletonBoneReferenceParent<'a> {
    pub(crate) key: SkeletonBoneKey,
    pub(crate) name: &'a str,
}

/// Read-only values borrowed from one installed skeleton's reference-local bone data.
///
/// Rotation remains the source quaternion; these values are neither an evaluated animation pose
/// nor a mesh/world-space transform. Retaining them does not authorize current picking or editing.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SkeletonBoneReferenceDetails<'a> {
    pub(crate) key: SkeletonBoneKey,
    pub(crate) name: &'a str,
    pub(crate) parent: Option<SkeletonBoneReferenceParent<'a>>,
    pub(crate) reference_local_translation: &'a [Real; 3],
    pub(crate) reference_local_rotation: &'a [Real; 4],
    pub(crate) reference_local_scale: &'a [Real; 3],
}
