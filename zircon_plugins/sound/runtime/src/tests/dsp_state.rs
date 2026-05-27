use super::*;

#[test]
fn dsp_bypass_wet_dry_delay_pan_phase_and_limiter_are_deterministic() {
    let mut wet_gain = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 0.0 }));
    wet_gain.wet = 0.25;
    assert_samples_near(&render_master_effect(wet_gain, &[1.0], 1), &[0.75, 0.75]);

    let mut bypass = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 0.0 }));
    bypass.bypass = true;
    assert_samples_near(&render_master_effect(bypass, &[1.0], 1), &[1.0, 1.0]);

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Delay(SoundDelayEffect {
                delay_frames: 1,
                feedback: 0.0,
            })),
            &[0.5, 0.0],
            2,
        ),
        &[0.0, 0.0, 0.5, 0.5],
    );

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::PanStereo(SoundPanStereoEffect {
                pan: 1.0,
                width: 1.0,
                left_gain: 1.0,
                right_gain: 1.0,
                invert_left_phase: true,
                invert_right_phase: false,
            })),
            &[0.5],
            1,
        ),
        &[-0.0, 0.5],
    );

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Limiter(SoundLimiterEffect {
                ceiling: 0.25,
            })),
            &[0.75],
            1,
        ),
        &[0.25, 0.25],
    );
}

#[test]
fn dsp_filter_reverb_waveshaper_and_modulation_effects_are_deterministic() {
    let low_pass = render_master_effect(
        test_effect(SoundEffectKind::Filter(SoundFilterEffect {
            mode: SoundFilterMode::LowPass,
            cutoff_hz: 1_000.0,
            resonance: 0.0,
            gain_db: 0.0,
        })),
        &[1.0, 0.0],
        2,
    );
    assert!(low_pass[0] > 0.0 && low_pass[0] < 0.2);
    assert!(low_pass[2] > low_pass[0] && low_pass[2] < 0.1);

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Reverb(SoundReverbEffect {
                room_size: 0.5,
                damping: 0.5,
                pre_delay_frames: 1,
                tail_frames: 2,
            })),
            &[0.5, 0.0],
            2,
        ),
        &[0.5, 0.5, 0.375, 0.375],
    );

    let shaped = render_master_effect(
        test_effect(SoundEffectKind::WaveShaper(SoundWaveShaperEffect {
            drive: 2.0,
        })),
        &[0.5],
        1,
    );
    assert!(shaped[0] > 0.5 && shaped[0] <= 1.0);

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Flanger(SoundFlangerEffect {
                delay_frames: 0,
                depth_frames: 0,
                rate_hz: 0.0,
                feedback: 0.0,
            })),
            &[0.25],
            1,
        ),
        &[0.375, 0.375],
    );

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Phaser(SoundPhaserEffect {
                rate_hz: 0.0,
                depth: 1.0,
                feedback: 0.0,
                phase_offset: 0.25,
            })),
            &[0.5],
            1,
        ),
        &[0.0, 0.0],
    );

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Chorus(SoundChorusEffect {
                voices: 1,
                delay_frames: 0,
                depth_frames: 0,
                rate_hz: 0.0,
            })),
            &[0.25],
            1,
        ),
        &[0.375, 0.375],
    );
}

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
fn low_pass_filter_keeps_state_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/stateful-filter.wav", &[1.0]));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Filter(SoundFilterEffect {
                mode: SoundFilterMode::LowPass,
                cutoff_hz: 1_000.0,
                resonance: 0.0,
                gain_db: 0.0,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let first = sound.render_mix(1).unwrap().samples;
    let second = sound.render_mix(1).unwrap().samples;

    assert!(first[0] > 0.0 && first[0] < 0.05);
    assert!(second[0] > first[0]);
    assert_sample_near(first[0], first[1]);
    assert_sample_near(second[0], second[1]);
}

#[test]
fn high_pass_filter_rejects_dc() {
    let sound = DefaultSoundManager::default();
    let clip =
        sound.insert_clip_for_test(test_clip("res://sound/high-pass-dc.wav", &vec![0.5; 128]));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Filter(SoundFilterEffect {
                mode: SoundFilterMode::HighPass,
                cutoff_hz: 1_000.0,
                resonance: 0.0,
                gain_db: 0.0,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let mix = sound.render_mix(128).unwrap().samples;
    let first_left = mix[0].abs();
    let last_left = mix[mix.len() - 2].abs();

    assert!(first_left > 0.25);
    assert!(last_left < first_left * 0.1);
}

#[test]
fn shelf_filter_uses_gain_db() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/low-shelf-gain.wav",
        &vec![0.25; 512],
    ));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Filter(SoundFilterEffect {
                mode: SoundFilterMode::LowShelf,
                cutoff_hz: 1_000.0,
                resonance: 0.0,
                gain_db: 6.0,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let mix = sound.render_mix(512).unwrap().samples;
    let settled_left = mix[mix.len() - 2];

    assert!(settled_left > 0.35);
    assert_sample_near(settled_left, mix[mix.len() - 1]);
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

#[test]
fn production_dsp_rejects_invalid_effect_parameters_before_render() {
    let sound = DefaultSoundManager::default();

    let mut invalid_wet = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 1.0 }));
    invalid_wet.wet = f32::NAN;
    assert!(sound
        .add_or_update_effect(SoundTrackId::master(), invalid_wet)
        .unwrap_err()
        .to_string()
        .contains("wet mix"));

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Delay(SoundDelayEffect {
                delay_frames: 1,
                feedback: f32::INFINITY,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("feedback"));

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Reverb(SoundReverbEffect {
                room_size: 1.25,
                damping: 0.5,
                pre_delay_frames: 1,
                tail_frames: 2,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("room size"));

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::WaveShaper(SoundWaveShaperEffect {
                drive: -0.1,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("drive"));

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::PanStereo(SoundPanStereoEffect {
                pan: 1.5,
                width: 1.0,
                left_gain: 1.0,
                right_gain: 1.0,
                invert_left_phase: false,
                invert_right_phase: false,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("stereo pan"));
}

#[test]
fn effect_update_revalidates_sidechain_track_references_and_cycles() {
    let sound = DefaultSoundManager::default();
    let key = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(key, "Key"))
        .unwrap();

    assert!(matches!(
        sound
            .add_or_update_effect(
                SoundTrackId::master(),
                test_effect(SoundEffectKind::Compressor(SoundCompressorEffect {
                    threshold_db: -12.0,
                    ratio: 2.0,
                    attack_ms: 1.0,
                    release_ms: 10.0,
                    makeup_gain_db: 0.0,
                    sidechain: Some(SoundSidechainInput {
                        track: SoundTrackId::new(999),
                        pre_effects: true,
                    }),
                })),
            )
            .unwrap_err(),
        SoundError::UnknownTrack { .. }
    ));

    assert!(sound
        .add_or_update_effect(
            key,
            test_effect(SoundEffectKind::Compressor(SoundCompressorEffect {
                threshold_db: -12.0,
                ratio: 2.0,
                attack_ms: 1.0,
                release_ms: 10.0,
                makeup_gain_db: 0.0,
                sidechain: Some(SoundSidechainInput {
                    track: key,
                    pre_effects: false,
                }),
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("post-effect sidechain"));

    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Compressor(SoundCompressorEffect {
                threshold_db: -12.0,
                ratio: 2.0,
                attack_ms: 1.0,
                release_ms: 10.0,
                makeup_gain_db: 0.0,
                sidechain: Some(SoundSidechainInput {
                    track: key,
                    pre_effects: true,
                }),
            })),
        )
        .unwrap();
}

fn render_master_effect(
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
