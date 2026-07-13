use std::collections::BTreeMap;

use zircon_runtime::core::framework::animation::AnimationSkeletonAsset;
use zircon_runtime::core::framework::animation::AnimationTargetId;
use zircon_runtime::core::framework::scene::EntityPath;

use super::{AnimationClipCompileError, TargetSlot, TargetTable, TargetTableError};

/// Skeleton-scoped target ownership compiled once when the skeleton is loaded.
///
/// Dense slots intentionally remain private to this table so a slot from one
/// skeleton cannot be used with another skeleton by public API consumers.
#[derive(Clone, Debug)]
pub struct SkeletonTargetTable {
    targets: TargetTable<usize>,
    bone_slots: Box<[TargetSlot]>,
    bone_names: Box<[String]>,
    bone_name_indices: BTreeMap<String, Option<usize>>,
    bone_paths: Box<[String]>,
    target_ids: Box<[AnimationTargetId]>,
}

impl SkeletonTargetTable {
    pub fn compile(skeleton: &AnimationSkeletonAsset) -> Result<Self, AnimationClipCompileError> {
        let mut targets = TargetTable::new();
        let mut bone_slots = Vec::with_capacity(skeleton.bones.len());
        let mut target_ids = Vec::with_capacity(skeleton.bones.len());
        let mut bone_paths = Vec::with_capacity(skeleton.bones.len());
        let mut bone_name_indices = BTreeMap::new();

        for bone_index in 0..skeleton.bones.len() {
            let path = skeleton_bone_path(skeleton, bone_index)?;
            let target_id = AnimationTargetId::from_path(&path);
            let slot = targets
                .bind(target_id, bone_index)
                .map_err(|error| match error {
                    TargetTableError::ConflictingBinding { .. } => {
                        AnimationClipCompileError::DuplicateTarget { target_id }
                    }
                    TargetTableError::CapacityExceeded => {
                        AnimationClipCompileError::TargetCapacityExceeded
                    }
                })?;
            bone_slots.push(slot);
            bone_paths.push(path.as_str().to_string());
            target_ids.push(target_id);
            bone_name_indices
                .entry(skeleton.bones[bone_index].name.clone())
                .and_modify(|index| *index = None)
                .or_insert(Some(bone_index));
        }

        Ok(Self {
            targets,
            bone_slots: bone_slots.into_boxed_slice(),
            bone_names: skeleton
                .bones
                .iter()
                .map(|bone| bone.name.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            bone_name_indices,
            bone_paths: bone_paths.into_boxed_slice(),
            target_ids: target_ids.into_boxed_slice(),
        })
    }

    pub fn len(&self) -> usize {
        self.bone_slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bone_slots.is_empty()
    }

    pub fn target_id_for_bone(&self, bone_index: usize) -> Option<AnimationTargetId> {
        self.target_ids.get(bone_index).copied()
    }

    pub fn bone_index_for_target(&self, target_id: AnimationTargetId) -> Option<usize> {
        let slot = self.targets.slot(target_id)?;
        self.bone_index_for_slot(slot)
    }

    pub(crate) fn slot_for_target(&self, target_id: AnimationTargetId) -> Option<TargetSlot> {
        self.targets.slot(target_id)
    }

    pub(crate) fn target_id_for_slot(&self, slot: TargetSlot) -> Option<AnimationTargetId> {
        self.target_ids.get(slot.index() as usize).copied()
    }

    pub(crate) fn bone_index_for_slot(&self, slot: TargetSlot) -> Option<usize> {
        self.targets.target(slot).copied()
    }

    pub(crate) fn bone_index_for_unique_name(&self, name: &str) -> Option<usize> {
        self.bone_name_indices.get(name).copied().flatten()
    }

    pub(crate) fn bone_paths(&self) -> &[String] {
        &self.bone_paths
    }

    pub(crate) fn bone_path_for_index(&self, bone_index: usize) -> Option<&str> {
        self.bone_paths.get(bone_index).map(String::as_str)
    }

    pub(crate) fn resolve_unique_bone_name(
        &self,
        track_index: usize,
        target: &str,
    ) -> Result<TargetSlot, AnimationClipCompileError> {
        let mut matches = self
            .bone_names
            .iter()
            .enumerate()
            .filter_map(|(bone_index, name)| (name == target).then_some(bone_index));
        let Some(bone_index) = matches.next() else {
            return Err(AnimationClipCompileError::UnresolvedTrack {
                track_index,
                target: target.to_string(),
            });
        };
        if matches.next().is_some() {
            return Err(AnimationClipCompileError::AmbiguousTrack {
                track_index,
                target: target.to_string(),
            });
        }
        Ok(self.bone_slots[bone_index])
    }
}

fn skeleton_bone_path(
    skeleton: &AnimationSkeletonAsset,
    bone_index: usize,
) -> Result<EntityPath, AnimationClipCompileError> {
    let mut visited = vec![false; skeleton.bones.len()];
    let mut segments = Vec::new();
    let mut current = bone_index;

    loop {
        if visited[current] {
            return Err(AnimationClipCompileError::ParentCycle {
                bone_index: current,
            });
        }
        visited[current] = true;
        let bone = &skeleton.bones[current];
        validate_bone_name(current, &bone.name)?;
        segments.push(bone.name.clone());

        let Some(parent_index) = bone.parent_index.map(|parent| parent as usize) else {
            break;
        };
        if parent_index >= skeleton.bones.len() {
            return Err(AnimationClipCompileError::InvalidParentIndex {
                bone_index: current,
                parent_index,
            });
        }
        current = parent_index;
    }

    segments.reverse();
    EntityPath::new(segments).map_err(|_| AnimationClipCompileError::EmptyBoneName { bone_index })
}

fn validate_bone_name(bone_index: usize, name: &str) -> Result<(), AnimationClipCompileError> {
    if name.trim().is_empty() {
        return Err(AnimationClipCompileError::EmptyBoneName { bone_index });
    }
    if name != name.trim() || name.contains('/') {
        return Err(AnimationClipCompileError::NonCanonicalBoneName {
            bone_index,
            name: name.to_string(),
        });
    }
    Ok(())
}
