use crate::asset::AnimationSequenceAsset;
use crate::core::framework::animation::{
    AnimationResult, AnimationSequenceApplyReport, AnimationTrackPath,
};
use crate::core::math::Real;
use crate::scene::world::World;

use super::channel_sample::AnimationChannelSampleExt;
use super::conversion::scene_property_value_from_channel;
use super::target::resolve_sequence_target_id;
use super::time::resolve_sequence_sample_time;

pub fn apply_sequence_to_world(
    world: &mut World,
    sequence: &AnimationSequenceAsset,
    time_seconds: Real,
    looping: bool,
) -> AnimationResult<AnimationSequenceApplyReport> {
    let mut report = AnimationSequenceApplyReport::default();
    let sample_time =
        resolve_sequence_sample_time(sequence.duration_seconds, time_seconds, looping);

    for binding in &sequence.bindings {
        let Some(entity) = binding
            .target_id
            .as_deref()
            .and_then(|target_id| resolve_sequence_target_id(world, target_id))
            .or_else(|| world.get_entity_by_path(&binding.entity_path))
        else {
            report
                .missing_tracks
                .extend(binding.tracks.iter().map(|track| {
                    AnimationTrackPath::new(
                        binding.entity_path.clone(),
                        track.property_path.clone(),
                    )
                }));
            continue;
        };

        for track in &binding.tracks {
            let track_path =
                AnimationTrackPath::new(binding.entity_path.clone(), track.property_path.clone());
            let Some(sample) = track.channel.sample(sample_time) else {
                report.missing_tracks.push(track_path);
                continue;
            };
            let value = scene_property_value_from_channel(&sample)?;
            match world.set_property(entity, &track.property_path, value) {
                Ok(_) => report.applied_tracks.push(track_path),
                Err(_) => report.missing_tracks.push(track_path),
            }
        }
    }

    Ok(report)
}
