use super::super::super::super::*;
use super::super::super::support::render_master_effect;

#[test]
fn dsp_wet_dry_mix_is_deterministic() {
    let mut wet_gain = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 0.0 }));
    wet_gain.wet = 0.25;
    assert_samples_near(&render_master_effect(wet_gain, &[1.0], 1), &[0.75, 0.75]);
}
