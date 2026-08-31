#[cfg(test)]
mod performance_tests;

use std::collections::HashMap;

use crate::core::framework::animation::{
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset,
};
use crate::core::framework::animation::{
    AnimationError, AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource, AnimationResult,
};
use crate::core::math::{Quat, Real, Transform, Vec3};

use super::sampling::{
    quaternion_array_is_normalizable, real_array_is_finite, resolve_sample_time, sample_quaternion,
    sample_vec3,
};
use crate::animation::sequence::AnimationChannelSampleExt;

pub(super) fn sample_clip_pose(
    skeleton: &AnimationSkeletonAsset,
    clip: &AnimationClipAsset,
    time_seconds: Real,
    looping: bool,
) -> AnimationResult<AnimationPoseOutput> {
    let sample_time = resolve_sample_time(clip.duration_seconds, time_seconds, looping);
    let mut bones = skeleton
        .bones
        .iter()
        .map(animation_pose_bone_from_skeleton)
        .collect::<AnimationResult<Vec<_>>>()?;
    let track_bone_index = ClipTrackBoneIndex::new(skeleton, &clip.tracks);

    for track in &clip.tracks {
        let Some(bone_index) = track_bone_index.resolve(track) else {
            continue;
        };
        let Some(bone) = bones.get_mut(bone_index) else {
            continue;
        };
        if let Some(sample) = track.translation.sample(sample_time) {
            bone.local_transform.translation = sample_vec3(&sample)?;
        }
        if let Some(sample) = track.rotation.sample(sample_time) {
            bone.local_transform.rotation = sample_quaternion(&sample)?;
        }
        if let Some(sample) = track.scale.sample(sample_time) {
            bone.local_transform.scale = sample_vec3(&sample)?;
        }
    }

    Ok(AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones,
    })
}

fn animation_pose_bone_from_skeleton(
    bone: &AnimationSkeletonBoneAsset,
) -> AnimationResult<AnimationPoseBone> {
    if !real_array_is_finite(&bone.local_translation) {
        return Err(AnimationError::NonFiniteSkeletonBind {
            bone: bone.name.clone(),
            field: "translation",
        });
    }
    if !real_array_is_finite(&bone.local_rotation) {
        return Err(AnimationError::NonFiniteSkeletonBind {
            bone: bone.name.clone(),
            field: "rotation",
        });
    }
    if !quaternion_array_is_normalizable(&bone.local_rotation) {
        return Err(AnimationError::ZeroLengthSkeletonBindRotation {
            bone: bone.name.clone(),
        });
    }
    if !real_array_is_finite(&bone.local_scale) {
        return Err(AnimationError::NonFiniteSkeletonBind {
            bone: bone.name.clone(),
            field: "scale",
        });
    }

    Ok(AnimationPoseBone {
        name: bone.name.clone(),
        local_transform: Transform {
            translation: Vec3::from_array(bone.local_translation),
            rotation: Quat::from_array(bone.local_rotation).normalize(),
            scale: Vec3::from_array(bone.local_scale),
        },
    })
}

struct ClipTrackBoneIndex<'skeleton> {
    bone_names: HashMap<&'skeleton str, usize>,
    bone_paths: HashMap<String, usize>,
}

impl<'skeleton> ClipTrackBoneIndex<'skeleton> {
    fn new(
        skeleton: &'skeleton AnimationSkeletonAsset,
        tracks: &[AnimationClipBoneTrackAsset],
    ) -> Self {
        let mut bone_names = HashMap::with_capacity(skeleton.bones.len());
        for (index, bone) in skeleton.bones.iter().enumerate() {
            bone_names.entry(bone.name.as_str()).or_insert(index);
        }

        let needs_path_index = tracks.iter().any(|track| {
            track
                .target_id
                .as_deref()
                .map(str::trim)
                .filter(|target_id| !target_id.is_empty())
                .is_some_and(|target_id| !bone_names.contains_key(target_id))
        });
        let mut bone_paths = HashMap::with_capacity(if needs_path_index {
            skeleton.bones.len()
        } else {
            0
        });
        if needs_path_index {
            for index in 0..skeleton.bones.len() {
                if let Some(path) = skeleton_bone_path(skeleton, index) {
                    bone_paths.entry(path).or_insert(index);
                }
            }
        }

        Self {
            bone_names,
            bone_paths,
        }
    }

    fn resolve(&self, track: &AnimationClipBoneTrackAsset) -> Option<usize> {
        if let Some(target_id) = track
            .target_id
            .as_deref()
            .map(str::trim)
            .filter(|target_id| !target_id.is_empty())
        {
            if let Some(index) = self.bone_names.get(target_id) {
                return Some(*index);
            }
            if let Some(index) = self.bone_paths.get(target_id) {
                return Some(*index);
            }
        }
        self.bone_names.get(track.bone_name.as_str()).copied()
    }
}

fn skeleton_bone_path(skeleton: &AnimationSkeletonAsset, index: usize) -> Option<String> {
    let bone = skeleton.bones.get(index)?;
    let mut segments = vec![bone.name.clone()];
    let mut parent = bone.parent_index;
    while let Some(parent_index) = parent {
        let parent_bone = skeleton.bones.get(parent_index as usize)?;
        segments.push(parent_bone.name.clone());
        parent = parent_bone.parent_index;
    }
    segments.reverse();
    Some(segments.join("/"))
}
