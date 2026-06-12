use super::super::super::super::super::*;

pub(super) fn play_software_null_test_clip(sound: &DefaultSoundManager) {
    let clip = sound.insert_clip_for_test(test_clip("res://sound/null-output.wav", &[0.25, 0.5]));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
}
