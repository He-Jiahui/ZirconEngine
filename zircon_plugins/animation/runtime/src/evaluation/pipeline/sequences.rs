use zircon_runtime::core::framework::animation::AnimationSequenceAsset;
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::LevelSystem;

pub(super) fn apply_loaded_sequences(
    level: &LevelSystem,
    loaded_sequences: &[(AnimationSequenceAsset, Real, bool)],
) {
    level.with_world_mut(|world| {
        for (sequence, time_seconds, looping) in loaded_sequences {
            let _ =
                crate::sequence::apply_sequence_to_world(world, sequence, *time_seconds, *looping);
        }
    });
}
