use super::super::super::*;

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
