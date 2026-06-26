use super::super::super::super::super::*;

pub(crate) fn play_output_test_clip(sound: &DefaultSoundManager) {
    let clip = sound.insert_clip_for_test(test_clip("res://sound/output.wav", &[0.25, 0.5]));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
}
