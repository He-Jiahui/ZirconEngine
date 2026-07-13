use std::collections::BTreeMap;

use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationClipBoneTrackAsset,
};

use super::super::skeleton::DerivedSkeletonAsset;
use super::channels::{constant_quaternion_channel, constant_vec3_channel};

#[derive(Clone)]
pub(super) struct DerivedClipTrack {
    pub(super) translation: AnimationChannelAsset,
    pub(super) rotation: AnimationChannelAsset,
    pub(super) scale: AnimationChannelAsset,
}

pub(super) fn default_clip_tracks_for_skeleton(
    skeleton: &DerivedSkeletonAsset,
) -> BTreeMap<String, DerivedClipTrack> {
    skeleton
        .joints
        .values()
        .map(|joint| {
            (
                joint.bone_name.clone(),
                DerivedClipTrack {
                    translation: constant_vec3_channel(joint.local_translation),
                    rotation: constant_quaternion_channel(joint.local_rotation),
                    scale: constant_vec3_channel(joint.local_scale),
                },
            )
        })
        .collect()
}

pub(super) fn into_clip_bone_tracks(
    tracks: BTreeMap<String, DerivedClipTrack>,
) -> Vec<AnimationClipBoneTrackAsset> {
    tracks
        .into_iter()
        .map(|(bone_name, track)| AnimationClipBoneTrackAsset {
            bone_name,
            target_id: None,
            translation: track.translation,
            rotation: track.rotation,
            scale: track.scale,
        })
        .collect()
}
