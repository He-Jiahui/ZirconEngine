use super::super::super::super::*;
use super::super::super::support::render_master_effect;

#[test]
fn phaser_effect_is_deterministic() {
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
}
