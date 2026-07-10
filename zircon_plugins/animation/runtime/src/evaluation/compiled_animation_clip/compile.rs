use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::asset::AnimationClipBoneTrackAsset;
use zircon_runtime::core::framework::animation::AnimationTargetId;
use zircon_runtime::core::framework::scene::EntityPath;

use super::super::{AnimationClipCompileError, CompiledClipTrack, SkeletonTargetTable, TargetSlot};
use super::CompiledAnimationClip;

impl CompiledAnimationClip {
    pub fn compile(
        target_table: Arc<SkeletonTargetTable>,
        source_tracks: &[AnimationClipBoneTrackAsset],
    ) -> Result<Self, AnimationClipCompileError> {
        let mut first_track_by_slot = BTreeMap::<TargetSlot, usize>::new();
        let mut tracks = Vec::with_capacity(source_tracks.len());

        for (track_index, track) in source_tracks.iter().enumerate() {
            let target = resolve_track_target(track_index, track, &target_table)?;
            if let Some(first_track_index) = first_track_by_slot.insert(target, track_index) {
                let target_id = target_table
                    .target_id_for_slot(target)
                    .ok_or(AnimationClipCompileError::MissingResolvedTarget { track_index })?;
                return Err(AnimationClipCompileError::DuplicateTrackTarget {
                    first_track_index,
                    duplicate_track_index: track_index,
                    target_id,
                });
            }
            tracks.push(CompiledClipTrack {
                target,
                translation: track.translation.clone(),
                rotation: track.rotation.clone(),
                scale: track.scale.clone(),
            });
        }

        Ok(Self {
            target_table,
            tracks,
        })
    }
}

fn resolve_track_target(
    track_index: usize,
    track: &AnimationClipBoneTrackAsset,
    target_table: &SkeletonTargetTable,
) -> Result<TargetSlot, AnimationClipCompileError> {
    if let Some(target) = track.target_id.as_deref() {
        let path = canonical_track_path(track_index, target)?;
        let target_id = AnimationTargetId::from_path(&path);
        return target_table.slot_for_target(target_id).ok_or_else(|| {
            AnimationClipCompileError::UnresolvedTrack {
                track_index,
                target: target.to_string(),
            }
        });
    }

    let target = track.bone_name.as_str();
    validate_track_leaf(track_index, target)?;
    target_table.resolve_unique_bone_name(track_index, target)
}

fn canonical_track_path(
    track_index: usize,
    target: &str,
) -> Result<EntityPath, AnimationClipCompileError> {
    let path = EntityPath::parse(target).map_err(|_| non_canonical_track(track_index, target))?;
    if path.as_str() != target {
        return Err(non_canonical_track(track_index, target));
    }
    Ok(path)
}

fn validate_track_leaf(track_index: usize, target: &str) -> Result<(), AnimationClipCompileError> {
    if target.trim().is_empty() || target != target.trim() || target.contains('/') {
        return Err(non_canonical_track(track_index, target));
    }
    Ok(())
}

fn non_canonical_track(track_index: usize, target: &str) -> AnimationClipCompileError {
    AnimationClipCompileError::NonCanonicalTrackTarget {
        track_index,
        target: target.to_string(),
    }
}
