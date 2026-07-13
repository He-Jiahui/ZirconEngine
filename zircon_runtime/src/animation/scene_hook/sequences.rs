use crate::core::framework::animation::AnimationSequenceAsset;
use crate::core::math::Real;
use crate::scene::LevelSystem;

pub(super) fn apply_loaded_sequences(
    level: &LevelSystem,
    loaded_sequences: &[(AnimationSequenceAsset, Real, bool)],
) {
    level.with_world_mut(|world| {
        for (sequence, time_seconds, looping) in loaded_sequences {
            let _ = crate::animation::sequence::apply_sequence_to_world(
                world,
                sequence,
                *time_seconds,
                *looping,
            );
        }
    });
}
