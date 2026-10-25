use zircon_runtime::core::framework::animation::AnimationSkeletonBoneAsset;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum SkeletonReferenceHierarchyError {
    #[error("skeleton has {bone_count} bones, exceeding the supported joint slot range")]
    TooManyBones { bone_count: usize },
    #[error("skeleton bone {bone_index} has parent {parent_index} outside {bone_count} slots")]
    ParentOutOfBounds {
        bone_index: usize,
        parent_index: u32,
        bone_count: usize,
    },
    #[error("skeleton parent cycle prevents reaching bone {bone_index} from a root")]
    ParentCycle { bone_index: usize },
}

#[derive(Debug)]
pub(super) struct SkeletonReferenceHierarchy {
    pub(super) parents: Vec<Option<usize>>,
    pub(super) children: Vec<Vec<usize>>,
    pub(super) preorder: Vec<usize>,
    pub(super) depths: Vec<usize>,
}

impl SkeletonReferenceHierarchy {
    pub(super) fn build(
        bones: &[AnimationSkeletonBoneAsset],
    ) -> Result<Self, SkeletonReferenceHierarchyError> {
        let bone_count = bones.len();
        if u32::try_from(bone_count).is_err() {
            return Err(SkeletonReferenceHierarchyError::TooManyBones { bone_count });
        }
        let mut parents = Vec::with_capacity(bone_count);
        let mut children = vec![Vec::new(); bone_count];
        let mut roots = Vec::new();
        for (bone_index, bone) in bones.iter().enumerate() {
            let parent = match bone.parent_index {
                Some(parent_index) => Some(
                    usize::try_from(parent_index)
                        .ok()
                        .filter(|&index| index < bone_count)
                        .ok_or(SkeletonReferenceHierarchyError::ParentOutOfBounds {
                            bone_index,
                            parent_index,
                            bone_count,
                        })?,
                ),
                None => None,
            };
            parents.push(parent);
            if let Some(parent) = parent {
                children[parent].push(bone_index);
            } else {
                roots.push(bone_index);
            }
        }

        // A valid forest visits each slot once. Components without a root contain a parent cycle.
        // Keep the traversal iterative; an imported hierarchy must not consume the call stack.
        let mut stack: Vec<_> = roots.into_iter().rev().map(|index| (index, 0)).collect();
        let mut visited = vec![false; bone_count];
        let mut depths = vec![0; bone_count];
        let mut preorder = Vec::with_capacity(bone_count);
        while let Some((index, depth)) = stack.pop() {
            visited[index] = true;
            depths[index] = depth;
            preorder.push(index);
            stack.extend(
                children[index]
                    .iter()
                    .rev()
                    .map(|&child| (child, depth + 1)),
            );
        }
        if let Some(bone_index) = visited.iter().position(|&visited| !visited) {
            return Err(SkeletonReferenceHierarchyError::ParentCycle { bone_index });
        }
        Ok(Self {
            parents,
            children,
            preorder,
            depths,
        })
    }
}
