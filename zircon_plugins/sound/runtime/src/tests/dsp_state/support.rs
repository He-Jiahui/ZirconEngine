use super::super::*;

pub(super) fn render_master_effect(
    effect: SoundEffectDescriptor,
    mono_samples: &[f32],
    frames: usize,
) -> Vec<f32> {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/effect.wav", mono_samples));
    sound
        .add_or_update_effect(SoundTrackId::master(), effect)
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    sound.render_mix(frames).unwrap().samples
}
