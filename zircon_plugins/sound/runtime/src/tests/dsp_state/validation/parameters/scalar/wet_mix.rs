use super::super::super::super::super::*;

#[test]
fn production_dsp_rejects_non_finite_wet_mix_before_render() {
    let sound = DefaultSoundManager::default();

    let mut invalid_wet = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 1.0 }));
    invalid_wet.wet = f32::NAN;
    assert!(sound
        .add_or_update_effect(SoundTrackId::master(), invalid_wet)
        .unwrap_err()
        .to_string()
        .contains("wet mix"));
}
