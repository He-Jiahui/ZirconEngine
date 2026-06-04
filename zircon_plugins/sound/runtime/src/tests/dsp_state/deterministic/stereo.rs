use super::super::super::*;
use super::super::support::render_master_effect;

#[test]
fn pan_phase_effect_is_deterministic() {
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
}
