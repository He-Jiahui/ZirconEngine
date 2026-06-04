use zircon_runtime::asset::{
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};

use super::sampling::{
    quaternion_array_is_normalizable, real_array_is_finite, resolve_sample_time, sample_quaternion,
    sample_vec3,
};
use crate::sequence::AnimationChannelSampleExt;

pub(super) fn sample_clip_pose(
    skeleton: &AnimationSkeletonAsset,
    clip: &AnimationClipAsset,
    time_seconds: Real,
    looping: bool,
) -> Result<AnimationPoseOutput, String> {
    let sample_time = resolve_sample_time(clip.duration_seconds, time_seconds, looping);
    let mut bones = skeleton
        .bones
        .iter()
        .map(animation_pose_bone_from_skeleton)
        .collect::<Result<Vec<_>, _>>()?;

    for track in &clip.tracks {
        let Some(bone_index) = resolve_clip_track_bone_index(skeleton, track) else {
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
) -> Result<AnimationPoseBone, String> {
    if !real_array_is_finite(&bone.local_translation) {
        return Err(format!(
            "non-finite skeleton bind translation for bone `{}`: {:?}",
            bone.name, bone.local_translation
        ));
    }
    if !real_array_is_finite(&bone.local_rotation) {
        return Err(format!(
            "non-finite skeleton bind rotation for bone `{}`: {:?}",
            bone.name, bone.local_rotation
        ));
    }
    if !quaternion_array_is_normalizable(&bone.local_rotation) {
        return Err(format!(
            "zero-length skeleton bind rotation for bone `{}`: {:?}",
            bone.name, bone.local_rotation
        ));
    }
    if !real_array_is_finite(&bone.local_scale) {
        return Err(format!(
            "non-finite skeleton bind scale for bone `{}`: {:?}",
            bone.name, bone.local_scale
        ));
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
    while let Some(parent_index) = parent {
        let parent_bone = skeleton.bones.get(parent_index as usize)?;
        segments.push(parent_bone.name.clone());
        parent = parent_bone.parent_index;
    }
    segments.reverse();
    Some(segments.join("/"))
}
