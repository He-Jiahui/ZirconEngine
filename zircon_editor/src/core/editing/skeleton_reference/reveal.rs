use zircon_runtime::core::framework::animation::AnimationSkeletonBoneAsset;

use super::hierarchy::SkeletonReferenceHierarchy;
use super::SkeletonBoneKey;

/// A read-only reveal plan for one immutable skeleton payload and plain-name query.
///
/// The shared host still owns selection, expansion, scroll realization, focus and event context.
/// These keys do not identify a socket/virtual bone or authorize an edit or preview pick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SkeletonBoneReveal {
    Included {
        key: SkeletonBoneKey,
        /// Root-to-parent source keys, excluding the target and every unrelated branch.
        ancestors: Vec<SkeletonBoneKey>,
    },
    FilteredOut {
        key: SkeletonBoneKey,
    },
}

/// Rows and reveal must agree about both direct matches and matching descendants' ancestors.
pub(super) fn name_query_visibility(
    bones: &[AnimationSkeletonBoneAsset],
    hierarchy: &SkeletonReferenceHierarchy,
    query: &str,
) -> (Vec<bool>, Vec<bool>) {
    let query = query.to_lowercase();
    let filtering = !query.is_empty();
    let matches: Vec<_> = bones
        .iter()
        .map(|bone| !filtering || bone.name.to_lowercase().contains(&query))
        .collect();
    let mut included = matches.clone();
    if filtering {
        for &index in hierarchy.preorder.iter().rev() {
            if included[index] {
                if let Some(parent) = hierarchy.parents[index] {
                    included[parent] = true;
                }
            }
        }
    }
    (matches, included)
}

/// The validated forest and source slot make this iterative walk bounded by its known depth.
pub(super) fn ancestor_indices(hierarchy: &SkeletonReferenceHierarchy, index: usize) -> Vec<usize> {
    let mut ancestors = Vec::with_capacity(hierarchy.depths[index]);
    let mut parent = hierarchy.parents[index];
    while let Some(index) = parent {
        ancestors.push(index);
        parent = hierarchy.parents[index];
    }
    ancestors.reverse();
    ancestors
}
