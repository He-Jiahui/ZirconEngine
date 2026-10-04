//! 管理器同步剪辑采样保留作者轨道到骨名或路径的解析顺序；帧系统则使用编译目标表。
use std::collections::HashMap;

use zircon_runtime::core::framework::animation::{
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationError, AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource, AnimationResult,
};
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};

use super::sampling::{
    quaternion_array_is_normalizable, real_array_is_finite, resolve_sample_time, sample_quaternion,
    sample_vec3,
};
use crate::channel_sampling::AnimationChannelSampleExt;

const BONE_INDEX_MIN_PROJECTED_COMPARISONS: usize = 2_048;

/// 同步采样已取得的作者资产，返回值不发布到场景；未解析轨道保持绑定姿态。
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
    let track_bone_lookup = ClipTrackBoneLookup::new(skeleton, &clip.tracks);

    for track in &clip.tracks {
        let Some(bone_index) = track_bone_lookup.resolve(track) else {
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

fn should_index_bone_lookup(bone_count: usize, track_count: usize) -> bool {
    bone_count.saturating_mul(track_count) >= BONE_INDEX_MIN_PROJECTED_COMPARISONS
}

struct ClipTrackBoneLookup<'skeleton> {
    skeleton: &'skeleton AnimationSkeletonAsset,
    bone_names: Option<HashMap<&'skeleton str, usize>>,
    bone_paths: HashMap<String, usize>,
}

impl<'skeleton> ClipTrackBoneLookup<'skeleton> {
    fn new(
        skeleton: &'skeleton AnimationSkeletonAsset,
        tracks: &[AnimationClipBoneTrackAsset],
    ) -> Self {
        if !should_index_bone_lookup(skeleton.bones.len(), tracks.len()) {
            return Self {
                skeleton,
                bone_names: None,
                bone_paths: HashMap::new(),
            };
        }

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
            skeleton,
            bone_names: Some(bone_names),
            bone_paths,
        }
    }

    fn resolve(&self, track: &AnimationClipBoneTrackAsset) -> Option<usize> {
        let Some(bone_names) = self.bone_names.as_ref() else {
            return resolve_clip_track_bone_index(self.skeleton, track);
        };
        if let Some(target_id) = track
            .target_id
            .as_deref()
            .map(str::trim)
            .filter(|target_id| !target_id.is_empty())
        {
            if let Some(index) = bone_names.get(target_id).copied() {
                return Some(index);
            }
            if let Some(index) = self.bone_paths.get(target_id).copied() {
                return Some(index);
            }
        }
        bone_names.get(track.bone_name.as_str()).copied()
    }
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

fn resolve_clip_track_bone_index(
    skeleton: &AnimationSkeletonAsset,
    track: &AnimationClipBoneTrackAsset,
) -> Option<usize> {
    if let Some(target_id) = track
        .target_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        if let Some(index) = skeleton
            .bones
            .iter()
            .position(|bone| bone.name == target_id)
        {
            return Some(index);
        }
        if let Some(index) = skeleton.bones.iter().enumerate().find_map(|(index, _)| {
            (skeleton_bone_path(skeleton, index)? == target_id).then_some(index)
        }) {
            return Some(index);
        }
    }

    skeleton
        .bones
        .iter()
        .position(|bone| bone.name == track.bone_name)
}

fn skeleton_bone_path(skeleton: &AnimationSkeletonAsset, index: usize) -> Option<String> {
    let bone = skeleton.bones.get(index)?;
    let mut segments = vec![bone.name.clone()];
    let mut parent = bone.parent_index;
    // BUG: [CR-PLUGIN-ANIMATION-0006] 原始骨架有父链环且轨道请求路径解析时，此循环不会结束；管理器公开采样入口未先做拓扑校验；证据：manager.rs:149 和 ClipTrackBoneLookup 的调用。
    while let Some(parent_index) = parent {
        let parent_bone = skeleton.bones.get(parent_index as usize)?;
        segments.push(parent_bone.name.clone());
        parent = parent_bone.parent_index;
    }
    segments.reverse();
    Some(segments.join("/"))
}

#[cfg(test)]
#[path = "tests/pose_optimization_batch_20260830cl_tests.rs"]
mod optimization_batch_20260830cl_tests;
