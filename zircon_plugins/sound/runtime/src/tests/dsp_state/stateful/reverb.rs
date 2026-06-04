use super::super::super::*;

#[test]
fn reverb_effect_keeps_tail_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/stateful-reverb.wav", &[0.5]));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Reverb(SoundReverbEffect {
                room_size: 0.5,
                damping: 0.5,
                pre_delay_frames: 1,
                tail_frames: 2,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.375, 0.375]);
}
