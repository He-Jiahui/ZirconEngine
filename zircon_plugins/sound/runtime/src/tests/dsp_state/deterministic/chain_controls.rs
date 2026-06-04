use super::super::super::*;
use super::super::support::render_master_effect;

#[test]
fn dsp_wet_dry_mix_is_deterministic() {
    let mut wet_gain = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 0.0 }));
    wet_gain.wet = 0.25;
    assert_samples_near(&render_master_effect(wet_gain, &[1.0], 1), &[0.75, 0.75]);
}

#[test]
fn dsp_bypass_is_deterministic() {
    let mut bypass = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 0.0 }));
    bypass.bypass = true;
    assert_samples_near(&render_master_effect(bypass, &[1.0], 1), &[1.0, 1.0]);
}
