use super::*;

#[test]
fn delay_effect_keeps_tail_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/stateful-delay.wav", &[0.5]));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Delay(SoundDelayEffect {
                delay_frames: 1,
                feedback: 0.0,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 0.0]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
}

#[test]
fn convolution_effect_keeps_impulse_tail_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/stateful-ir.wav", &[1.0]));
    let impulse_response = SoundImpulseResponseId::new(501);
    sound
        .set_impulse_response(impulse_response, vec![0.5, 0.25])
        .unwrap();
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            SoundEffectDescriptor::new(
                SoundEffectId::new(501),
                "Stateful IR",
                SoundEffectKind::ConvolutionReverb(SoundConvolutionReverbEffect {
                    impulse_response,
                    fallback_to_algorithmic: false,
                    latency_frames: 1,
                }),
            ),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.25, 0.25]);
}

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

#[test]
fn mixer_snapshot_reports_effect_and_track_delay_latency() {
    let sound = DefaultSoundManager::default();
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Delay(SoundDelayEffect {
                delay_frames: 7,
                feedback: 0.0,
            })),
        )
        .unwrap();

    sound.render_mix(1).unwrap();
    assert_eq!(sound.mixer_snapshot().unwrap().latency_frames, 7);

    let mut master = SoundTrackDescriptor::master();
    master.controls.delay_frames = 11;
    sound.add_or_update_track(master).unwrap();
    sound.render_mix(1).unwrap();
    assert_eq!(sound.mixer_snapshot().unwrap().latency_frames, 11);
}

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

#[test]
fn phaser_lfo_phase_continues_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip =
        sound.insert_clip_for_test(test_clip("res://sound/stateful-phaser.wav", &[1.0, 1.0]));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Phaser(SoundPhaserEffect {
                rate_hz: 12_000.0,
                depth: 1.0,
                feedback: 0.0,
                phase_offset: 0.25,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 0.0]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
}

#[test]
fn compressor_release_envelope_continues_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/stateful-compressor.wav",
        &[1.0, 0.1],
    ));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Compressor(SoundCompressorEffect {
                threshold_db: -12.0,
                ratio: 20.0,
                attack_ms: 0.0,
                release_ms: 1000.0,
                makeup_gain_db: 0.0,
                sidechain: None,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let first = sound.render_mix(1).unwrap().samples;
    let second = sound.render_mix(1).unwrap().samples;

    assert!(first[0] < 0.5);
    assert!(second[0] < 0.05);
    assert_sample_near(second[0], second[1]);
}
