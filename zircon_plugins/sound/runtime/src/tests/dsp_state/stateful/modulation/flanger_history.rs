use super::super::super::super::*;

#[test]
fn modulated_delay_keeps_history_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/stateful-flanger.wav", &[0.5]));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Flanger(SoundFlangerEffect {
                delay_frames: 1,
                depth_frames: 0,
                rate_hz: 0.0,
                feedback: 0.0,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.25, 0.25]);
}
