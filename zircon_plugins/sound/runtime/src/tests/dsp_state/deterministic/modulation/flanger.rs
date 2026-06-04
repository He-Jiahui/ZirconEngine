use super::super::super::super::*;
use super::super::super::support::render_master_effect;

#[test]
fn flanger_effect_is_deterministic() {
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
}
