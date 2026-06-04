use super::super::super::*;
use super::super::support::render_master_effect;

#[test]
fn limiter_effect_is_deterministic() {
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
