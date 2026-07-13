use super::skeleton::DerivedSkeletonAsset;
use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::framework::animation::AnimationClipAsset;
use zircon_runtime_interface::resource::ResourceLocator;

mod channels;
mod tracks;

use channels::{
    map_animation_interpolation, quaternion_channel_from_samples, vec3_channel_from_samples,
};
use tracks::{default_clip_tracks_for_skeleton, into_clip_bone_tracks};

pub(super) fn derive_clip_asset(
    animation: &gltf::Animation<'_>,
    buffers: &[gltf::buffer::Data],
    skeleton: &DerivedSkeletonAsset,
    skeleton_locator: &ResourceLocator,
) -> Result<AnimationClipAsset, String> {
    let mut tracks = default_clip_tracks_for_skeleton(skeleton);

    let mut duration_seconds = 0.0_f32;
    for channel in animation.channels() {
        let target_node = channel.target().node().index();
        let Some(joint) = skeleton.joints.get(&target_node) else {
            continue;
        };
        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()].0));
        let times = reader
            .read_inputs()
            .ok_or_else(|| "gltf animation channel is missing keyframe times".to_string())?
            .collect::<Vec<_>>();
        if let Some(last_time) = times.last().copied() {
            duration_seconds = duration_seconds.max(last_time);
        }
        let interpolation = map_animation_interpolation(channel.sampler().interpolation());
        let track = tracks
            .get_mut(&joint.bone_name)
            .ok_or_else(|| format!("missing derived joint track for {}", joint.bone_name))?;

        match reader
            .read_outputs()
            .ok_or_else(|| "gltf animation channel is missing output values".to_string())?
        {
            gltf::animation::util::ReadOutputs::Translations(values) => {
                track.translation =
                    vec3_channel_from_samples(&times, &values.collect::<Vec<_>>(), interpolation)?;
            }
            gltf::animation::util::ReadOutputs::Rotations(values) => {
                track.rotation = quaternion_channel_from_samples(
                    &times,
                    &values.into_f32().collect::<Vec<_>>(),
                    interpolation,
                )?;
            }
            gltf::animation::util::ReadOutputs::Scales(values) => {
                track.scale =
                    vec3_channel_from_samples(&times, &values.collect::<Vec<_>>(), interpolation)?;
            }
            gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => {}
        }
    }

    Ok(AnimationClipAsset {
        name: animation.name().map(str::to_string),
        skeleton: AssetReference::from_locator(skeleton_locator.clone()),
        duration_seconds,
        tracks: into_clip_bone_tracks(tracks),
        event_tracks: Vec::new(),
    })
}
