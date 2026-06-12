use super::super::super::super::*;
use super::super::super::support::render_master_effect;

#[test]
fn dsp_bypass_is_deterministic() {
    let mut bypass = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 0.0 }));
    bypass.bypass = true;
    assert_samples_near(&render_master_effect(bypass, &[1.0], 1), &[1.0, 1.0]);
}
