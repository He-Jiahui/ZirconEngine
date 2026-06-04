use super::super::super::*;
use super::super::support::render_master_effect;

#[test]
fn delay_effect_is_deterministic() {
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
}
